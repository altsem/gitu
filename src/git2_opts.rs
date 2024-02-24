use crate::Res;
use git2;
use git2::Repository;

pub(crate) fn status(repo: &Repository) -> Res<git2::StatusOptions> {
    let mut opts = git2::StatusOptions::new();

    opts.include_untracked(
        repo.config()?
            .get_bool("status.showUntrackedFiles")
            .ok()
            .unwrap_or(true),
    );

    Ok(opts)
}
