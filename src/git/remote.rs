// TODO: implement `set_push_remote`, test it, then allow using these functions
// from the branch configuration menu
use git2::{Branch, Reference, Remote, Repository};

use crate::Res;

pub(crate) fn get_push_remote(repo: &Repository) -> Res<String> {
    let push_remote_cfg = head_push_remote_cfg(repo)?;
    let config = repo.config()?;
    match config.get_string(&push_remote_cfg) {
        Ok(v) => Ok(v),
        Err(e) if e.class() == git2::ErrorClass::Config => Ok("".into()),
        Err(e) => Err(e.into())
    }
}

pub(crate) fn set_push_remote(repo: &Repository, remote: Option<&Remote>) -> Res<()> {
    let push_remote_cfg = head_push_remote_cfg(repo)?;
    let mut config = repo.config()?;
    let remote = remote.map(|v| v.name()).flatten().unwrap_or("");
    config.set_str(&push_remote_cfg, remote)?;
    Ok(())
}

pub(crate) fn head_push_remote_cfg(repo: &Repository) -> Res<String> {
    let head = repo.head()?;
    let branch = if head.is_branch() {
        head
            .shorthand()
            .ok_or("Head branch name was not valid UTF-8")?
    } else {
        return Err("Head is not a branch".into());
    };
    let push_remote_cfg = format!("branch.{branch}.pushRemote");
    Ok(push_remote_cfg)
}

/// Set the remote and upstream of the head. Can't be a detached head, must be a
/// branch.
pub(crate) fn set_upstream(repo: &Repository, upstream: Option<&Reference>) -> Res<()> {
    let head = repo.head()?;
    let mut head = if head.is_branch() {
        Branch::wrap(head)
    } else {
        return Err("Head is not a branch".into());
    };

    let upstream = upstream.map(|r| r.shorthand()).flatten();
    match (head.set_upstream(upstream), upstream) {
        (Ok(()), _) => Ok(()),
        // `set_upstream` will error if there isn't an existing config for
        // the branch when we try to remove the config
        (Err(e), None) if e.class() == git2::ErrorClass::Config => Ok(()),
        (Err(e), _) => Err(e.into())
    }
}
