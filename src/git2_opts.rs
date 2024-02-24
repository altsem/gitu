use crate::Res;
use git2::{self, DiffOptions, PushOptions, RemoteCallbacks, Repository, StatusOptions};

pub(crate) fn status(repo: &Repository) -> Res<StatusOptions> {
    let mut opts = StatusOptions::new();

    opts.include_untracked(
        repo.config()?
            .get_bool("status.showUntrackedFiles")
            .ok()
            .unwrap_or(true),
    );

    Ok(opts)
}

pub(crate) fn diff(_repo: &Repository) -> Res<DiffOptions> {
    Ok(DiffOptions::new())
}

pub(crate) fn push(repo: &Repository) -> Res<PushOptions<'_>> {
    let mut opts = PushOptions::new();
    opts.remote_callbacks(remote_callbacks(repo));
    Ok(opts)
}

fn remote_callbacks(repo: &Repository) -> RemoteCallbacks<'_> {
    let mut callbacks = RemoteCallbacks::new();

    callbacks.credentials(|url, username_from_url, allowed_types| {
        auth_git2::GitAuthenticator::default().credentials(&repo.config()?)(
            url,
            username_from_url,
            allowed_types,
        )
    });
    callbacks
}
