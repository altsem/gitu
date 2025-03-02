use git2::{Branch, Remote, Repository};

use crate::Res;

pub(crate) fn get_upstream(repo: &Repository) -> Res<Option<Branch>> {
    let r = if repo.head()?.is_branch() {
        Branch::wrap(repo.head()?)
    } else {
        return Err("Head is not a branch".into());
    };
    match r.upstream() {
        Ok(v) => Ok(Some(v)),
        Err(e) if e.class() == git2::ErrorClass::Config => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// If the branch has an upstream, returns the remote name and branch name in that order.
/// Returns "." as remote if the current branch has no remote upstream.
///
/// Branch references would be used like this (in Magit)
///
/// // Remote branch
/// git … push -v origin feature-branch\:refs/heads/feature-branch
/// git … pull origin refs/heads/feature-branch
/// git … rebase --autostash origin/feature-branch
///
/// // Local branch
/// git … push -v . feature-branch\:refs/heads/main
/// git … pull . refs/heads/main
/// git … rebase --autostash main
pub(crate) fn get_upstream_components(repo: &Repository) -> Res<Option<(String, String)>> {
    let Some(upstream) = get_upstream(repo)? else {
        return Ok(None);
    };

    let branch = upstream
        .get()
        .shorthand()
        .ok_or("Branch name not utf-8")?
        .to_string();

    if upstream.get().is_remote() {
        let branch_full = upstream.get().name().ok_or("Branch name not utf-8")?;
        let remote = repo
            .branch_remote_name(branch_full)?
            .as_str()
            .ok_or("Remote name not utf-8")?
            .to_string();

        let remote_prefix = format!("{}/", remote);
        Ok(Some((remote, branch.replace(&remote_prefix, ""))))
    } else {
        Ok(Some((".".into(), branch)))
    }
}

pub(crate) fn get_upstream_shortname(repo: &Repository) -> Res<Option<String>> {
    let Some(upstream) = get_upstream(repo)? else {
        return Ok(None);
    };
    Ok(Some(
        upstream
            .get()
            .shorthand()
            .ok_or("Upstream ref not utf-8")?
            .into(),
    ))
}

pub(crate) fn get_push_remote(repo: &Repository) -> Res<Option<String>> {
    let push_remote_cfg = head_push_remote_cfg(repo)?;
    let config = repo.config()?;
    match config.get_string(&push_remote_cfg) {
        Ok(v) if v.is_empty() => Ok(None),
        Ok(v) => Ok(Some(v)),
        Err(e) if e.class() == git2::ErrorClass::Config => Ok(None),
        Err(e) => Err(e.into()),
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
            config.set_str(&push_remote_cfg, remote.name().ok_or("Invalid remote")?)?;
        }
    }
    Ok(())
}

pub(crate) fn head_push_remote_cfg(repo: &Repository) -> Res<String> {
    let head = repo.head()?;
    let branch = if head.is_branch() {
        head.shorthand()
            .ok_or("Head branch name was not valid UTF-8")?
    } else {
        return Err("Head is not a branch".into());
    };

    let push_remote_cfg = format!("branch.{branch}.pushRemote");
    let config = repo.config()?;
    // Check if pushRemote is configured
    if config.get_string(&push_remote_cfg).is_ok() {
        Ok(push_remote_cfg)
    } else {
        // Fallback to push.default if pushRemote is not found
        let push_default_cfg = "remote.pushDefault".to_string();
        if config.get_string(&push_default_cfg).is_ok() {
            Ok(push_default_cfg)
        } else {
            Err("Neither pushRemote nor push.default is configured".into())
        }
    }
}
