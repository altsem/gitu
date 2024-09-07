use git2::{Branch, Reference, Remote, Repository};

use crate::Res;

pub(crate) fn get_upstream(r: Reference) -> Res<Option<Branch>> {
    let r = if r.is_branch() {
        Branch::wrap(r)
    } else {
        return Err("Head is not a branch".into());
    };
    match r.upstream() {
        Ok(v) => Ok(Some(v)),
        Err(e) if e.class() == git2::ErrorClass::Config => Ok(None),
        Err(e) => Err(e.into())
    }
}

/// If the branch has an upstream, returns the remote name and branch name in that order.
pub(crate) fn get_upstream_components(repo: &Repository) -> Res<Option<(String, String)>> {
   let Some(upstream) = get_upstream(repo.head()?)? else {
       return Ok(None)
   };
    let branch_full = upstream.get().name().ok_or("Branch name not utf-8")?;
    let remote = repo.branch_remote_name(branch_full)?;
    let remote = remote.as_str().ok_or("Remote name not utf-8")?;
    Ok(Some((remote.into(), repo.head()?.shorthand().ok_or("Branch name not utf-8")?.into())))
}

pub(crate) fn get_push_remote(repo: &Repository) -> Res<Option<String>> {
    let push_remote_cfg = head_push_remote_cfg(repo)?;
    let config = repo.config()?;
    match config.get_string(&push_remote_cfg) {
        Ok(v) if v == "" => Ok(None),
        Ok(v) => Ok(Some(v)),
        Err(e) if e.class() == git2::ErrorClass::Config => Ok(None),
        Err(e) => Err(e.into())
    }
}

pub(crate) fn set_push_remote(repo: &Repository, remote: Option<&Remote>) -> Res<()> {
    let push_remote_cfg = head_push_remote_cfg(repo)?;
    let mut config = repo.config()?;
    match remote {
        None => {
            config.remove(&push_remote_cfg)?;
        }
        Some(remote) => {
            config.set_str(&push_remote_cfg, remote.name().ok_or_else(|| "Invalid remote")?)?;
        }
    }
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
pub(crate) fn set_upstream(repo: &Repository, upstream: Option<&str>) -> Res<()> {
    let head = repo.head()?;
    let mut head = if head.is_branch() {
        Branch::wrap(head)
    } else {
        return Err("Head is not a branch".into());
    };

    match (head.set_upstream(upstream), upstream) {
        (Ok(()), _) => Ok(()),
        // `set_upstream` will error if there isn't an existing config for
        // the branch when we try to remove the config
        (Err(e), None) if e.class() == git2::ErrorClass::Config => Ok(()),
        (Err(e), _) => Err(e.into())
    }
}

pub(crate) fn set_upstream_from_ref(repo: &Repository, upstream: Option<&Reference>) -> Res<()> {
    let upstream = upstream.map(|r| r.shorthand()).flatten();
    set_upstream(repo, upstream)
}
