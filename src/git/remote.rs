// TODO: implement `set_push_remote`, test it, then allow using these functions
// from the branch configuration menu
use git2::{Branch, Reference, Repository};

use crate::Res;

/// Set the remote and upstream of the head. Can't be a detached head, must be a
/// branch.
pub(crate) fn set_upstream(repo: &Repository, upstream: Option<&Reference>) -> Res<()> {
    let head = repo.head()?;

    if head.is_branch() {
        let mut head = Branch::wrap(head);
        let upstream = upstream.map(|r| r.shorthand()).flatten();
        match (head.set_upstream(upstream), upstream) {
            (Ok(()), _) => Ok(()),
            // `set_upstream` will error if there isn't an existing config for
            // the branch when we try to remove the config
            (Err(e), None) if e.class() == git2::ErrorClass::Config => Ok(()),
            (Err(e), _) => Err(e.into())
        }
    } else {
        Err("Head is not a branch".into())
    }
}
