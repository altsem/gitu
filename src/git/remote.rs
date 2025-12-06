use std::{ops::Deref, str};

use git2::{Branch, Remote, Repository};

use crate::{Res, git};

use super::{Error, Utf8Error};

pub(crate) fn get_upstream(repo: &Repository) -> Res<Option<Branch<'_>>> {
    get_branch_upstream(&git::get_current_branch(repo)?)
}

pub(crate) fn get_branch_upstream<'repo>(branch: &Branch<'repo>) -> Res<Option<Branch<'repo>>> {
    match branch.upstream() {
        Ok(v) => Ok(Some(v)),
        Err(e) if e.class() == git2::ErrorClass::Config => Ok(None),
        Err(e) => Err(Error::GetCurrentBranchUpstream(e)),
    }
}

pub(crate) fn get_remote_name(repo: &Repository, upstream: &Branch) -> Res<String> {
    let branch_full = str::from_utf8(upstream.get().name_bytes())
        .map_err(Utf8Error::Str)
        .map_err(Error::BranchNameUtf8)?;

    String::from_utf8(
        repo.branch_remote_name(branch_full)
            .map_err(Error::GetRemote)?
            .deref()
            .to_vec(),
    )
    .map_err(Utf8Error::String)
    .map_err(Error::RemoteNameUtf8)
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

    let branch = String::from_utf8(upstream.get().shorthand_bytes().to_vec())
        .map_err(Utf8Error::String)
        .map_err(Error::BranchNameUtf8)?;

    if upstream.get().is_remote() {
        let remote_name = get_remote_name(repo, &upstream)?;
        let remote_prefix = format!("{remote_name}/");
        Ok(Some((remote_name, branch.replace(&remote_prefix, ""))))
    } else {
        Ok(Some((".".into(), branch)))
    }
}

pub(crate) fn get_upstream_shortname(repo: &Repository) -> Res<Option<String>> {
    let Some(upstream) = get_upstream(repo)? else {
        return Ok(None);
    };
    Ok(Some(
        String::from_utf8(upstream.get().shorthand_bytes().to_vec())
            .map_err(Utf8Error::String)
            .map_err(Error::GetCurrentBranchUpstreamUtf8)?,
    ))
}

pub(crate) fn get_upstream_remote(repo: &Repository) -> Res<Option<String>> {
    let Some(upstream) = get_upstream(repo)? else {
        return Ok(None);
    };

    if upstream.get().is_remote() {
        let remote_name = get_remote_name(repo, &upstream)?;
        Ok(Some(remote_name))
    } else {
        Ok(None)
    }
}

pub(crate) fn get_push_remote(repo: &Repository) -> Res<Option<String>> {
    let push_remote_cfg = head_push_remote_cfg(repo)?;
    let config = repo.config().map_err(Error::ReadGitConfig)?;

    match config.get_entry(&push_remote_cfg) {
        Ok(entry) => Ok(Some(
            String::from_utf8(entry.value_bytes().to_vec())
                .map_err(Utf8Error::String)
                .map_err(Error::ReadGitConfigUtf8)?,
        )),
        Err(e) if e.class() == git2::ErrorClass::Config => get_default_push_remote(repo),
        Err(e) => Err(Error::ReadGitConfig(e)),
    }
}

pub(crate) fn get_default_push_remote(repo: &Repository) -> Res<Option<String>> {
    let push_default_cfg = "remote.pushDefault";
    let config = repo.config().map_err(Error::ReadGitConfig)?;

    match config.get_entry(push_default_cfg) {
        Ok(entry) => Ok(Some(
            String::from_utf8(entry.value_bytes().to_vec())
                .map_err(Utf8Error::String)
                .map_err(Error::ReadGitConfigUtf8)?,
        )),
        Err(e) if e.class() == git2::ErrorClass::Config => Ok(None),
        Err(e) => Err(Error::ReadGitConfig(e)),
    }
}

pub(crate) fn set_push_remote(repo: &Repository, remote: Option<&Remote>) -> Res<()> {
    let push_remote_cfg = head_push_remote_cfg(repo)?;
    let mut config = repo.config().map_err(Error::ReadGitConfig)?;
    match remote {
        None => {
            config
                .remove(&push_remote_cfg)
                .map_err(Error::DeleteGitConfig)?;
        }
        Some(remote) => {
            config
                .set_str(
                    &push_remote_cfg,
                    str::from_utf8(remote.name_bytes().ok_or(Error::RemoteHasNoName)?)
                        .map_err(Utf8Error::Str)
                        .map_err(Error::RemoteNameUtf8)?,
                )
                .map_err(Error::SetGitConfig)?;
        }
    }
    Ok(())
}

pub(crate) fn head_push_remote_cfg(repo: &Repository) -> Res<String> {
    let branch = git::get_current_branch_name(repo)?;
    let push_remote_cfg = format!("branch.{branch}.pushRemote");
    Ok(push_remote_cfg)
}
