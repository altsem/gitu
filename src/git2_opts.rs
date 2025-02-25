use crate::Res;
use git2::{Repository, StatusOptions};

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
