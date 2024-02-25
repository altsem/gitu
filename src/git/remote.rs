use crate::Res;
use git2::PushOptions;
use git2::RemoteCallbacks;
use git2::Repository;

pub(crate) fn push_upstream(repo: &Repository) -> Res<String> {
    let Ok(head) = repo.head() else {
        return Err(Box::new(PushError::NoHead));
    };
    let Ok(upstream) = repo.branch_upstream_remote(head.name().unwrap()) else {
        return Err(Box::new(PushError::NoUpstreamRemote));
    };

    let mut remote = repo.find_remote(upstream.as_str().unwrap()).unwrap();
    let mut out_name = String::new();
    let mut out_status: Option<String> = None;

    {
        let mut opts = PushOptions::new();
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|url, username_from_url, allowed_types| {
            auth_git2::GitAuthenticator::default().credentials(&repo.config()?)(
                url,
                username_from_url,
                allowed_types,
            )
        });
        callbacks.push_update_reference(|name, status| {
            out_name.push_str(name);
            out_status = status.map(|s| s.to_string());
            Ok(())
        });
        opts.remote_callbacks(callbacks);

        remote.push::<&str>(&[head.name().unwrap()], Some(&mut opts))?;
    }

    if let Some(status) = out_status {
        return Err(Box::new(PushError::ResponseStatus(status)));
    }

    Ok(format!("Pushed {} to {}", out_name, remote.name().unwrap()))
}

#[derive(Debug)]
pub(crate) enum PushError {
    NoHead,
    NoUpstreamRemote,
    ResponseStatus(String),
}

impl std::fmt::Display for PushError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            PushError::NoHead => "No head",
            PushError::NoUpstreamRemote => "No upstream remote",
            PushError::ResponseStatus(status) => status,
        })
    }
}

impl std::error::Error for PushError {}
