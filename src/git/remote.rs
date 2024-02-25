use crate::Res;
use git2::FetchOptions;
use git2::PushOptions;
use git2::RemoteCallbacks;
use git2::Repository;

pub(crate) fn fetch_upstream(repo: &Repository) -> Res<String> {
    let Ok(head) = repo.head() else {
        return Err(Box::new(RemoteError::NoHead));
    };
    let Ok(upstream) = repo.branch_upstream_remote(head.name().unwrap()) else {
        return Err(Box::new(RemoteError::NoUpstreamRemote));
    };

    fetch(repo, repo.find_remote(upstream.as_str().unwrap()).unwrap())
}

pub(crate) fn fetch_all(repo: &Repository) -> Res<String> {
    for remote in repo.remotes()?.iter().flatten() {
        fetch(repo, repo.find_remote(remote).unwrap())?;
    }

    Ok("Fetched all".to_string())
}

fn fetch(repo: &Repository, mut remote: git2::Remote<'_>) -> Res<String> {
    {
        let mut opts = FetchOptions::new();
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|url, username_from_url, allowed_types| {
            auth_git2::GitAuthenticator::default().credentials(&repo.config()?)(
                url,
                username_from_url,
                allowed_types,
            )
        });

        opts.remote_callbacks(callbacks);

        let refspec_array = &remote.fetch_refspecs()?;
        let refspecs = refspec_array.into_iter().flatten().collect::<Vec<_>>();
        remote.fetch::<&str>(&refspecs, Some(&mut opts), None)?;
    };

    Ok(format!("Fetched {}", remote.name().unwrap()))
}

pub(crate) fn push_upstream(repo: &Repository) -> Res<String> {
    let Ok(head) = repo.head() else {
        return Err(Box::new(RemoteError::NoHead));
    };
    let Ok(upstream) = repo.branch_upstream_remote(head.name().unwrap()) else {
        return Err(Box::new(RemoteError::NoUpstreamRemote));
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
        return Err(Box::new(RemoteError::ResponseStatus(status)));
    }

    Ok(format!("Pushed {} to {}", out_name, remote.name().unwrap()))
}

#[derive(Debug)]
pub(crate) enum RemoteError {
    NoHead,
    NoUpstreamRemote,
    ResponseStatus(String),
}

impl std::fmt::Display for RemoteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            RemoteError::NoHead => "No head",
            RemoteError::NoUpstreamRemote => "No upstream remote",
            RemoteError::ResponseStatus(status) => status,
        })
    }
}

impl std::error::Error for RemoteError {}
