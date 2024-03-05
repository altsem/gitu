use crate::Res;
use git2::{DiffOptions, Repository, StatusOptions};

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
    let mut diff_options = DiffOptions::new();
    diff_options.patience(true);
    Ok(diff_options)
}
