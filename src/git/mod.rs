use diff::Diff;
use git2::{Branch, Repository};
use itertools::Itertools;

use self::{commit::Commit, merge_status::MergeStatus, rebase_status::RebaseStatus};
use crate::{
    error::{Error, Utf8Error},
    Res,
};
use std::{
    fs,
    path::Path,
    process::Command,
    str::{self},
};

pub(crate) mod commit;
pub(crate) mod diff;
pub(crate) mod merge_status;
pub(crate) mod rebase_status;
pub(crate) mod remote;

// TODO Use only plumbing commands

pub(crate) fn rebase_status(repo: &Repository) -> Res<Option<RebaseStatus>> {
    let dir = repo.workdir().expect("No workdir");
    let mut rebase_onto_file = dir.to_path_buf();
    rebase_onto_file.push(".git/rebase-merge/onto");

    let mut rebase_head_name_file = dir.to_path_buf();
    rebase_head_name_file.push(".git/rebase-merge/head-name");

    match fs::read_to_string(&rebase_onto_file) {
        Ok(content) => {
            let onto_hash = content.trim().to_string();
            Ok(Some(RebaseStatus {
                onto: branch_name_lossy(dir, &onto_hash)?
                    .unwrap_or_else(|| onto_hash[..7].to_string()),
                head_name: fs::read_to_string(rebase_head_name_file)
                    .map_err(Error::ReadRebaseStatusFile)?
                    .trim()
                    .strip_prefix("refs/heads/")
                    .unwrap()
                    .to_string(),
                // TODO include log of 'done' items
            }))
        }
        Err(err) => {
            log::warn!(
                "Couldn't read {}, due to {}",
                rebase_onto_file.to_string_lossy(),
                err
            );
            Ok(None)
        }
    }
}

pub(crate) fn merge_status(repo: &Repository) -> Res<Option<MergeStatus>> {
    let dir = repo.workdir().expect("No workdir");
    let mut merge_head_file = dir.to_path_buf();
    merge_head_file.push(".git/MERGE_HEAD");

    match fs::read_to_string(&merge_head_file) {
        Ok(content) => {
            let head = content.trim().to_string();
            Ok(Some(MergeStatus {
                head: branch_name_lossy(dir, &head)?.unwrap_or(head[..7].to_string()),
            }))
        }
        Err(err) => {
            log::warn!(
                "Couldn't read {}, due to {}",
                merge_head_file.to_string_lossy(),
                err
            );
            Ok(None)
        }
    }
}

pub(crate) struct RevertStatus {
    pub head: String,
}

pub(crate) fn revert_status(repo: &Repository) -> Res<Option<RevertStatus>> {
    let dir = repo.workdir().expect("No workdir");
    let mut revert_head_file = dir.to_path_buf();
    revert_head_file.push(".git/REVERT_HEAD");

    match fs::read_to_string(&revert_head_file) {
        Ok(content) => {
            let head = content.trim().to_string();
            Ok(Some(RevertStatus {
                head: branch_name_lossy(dir, &head)?.unwrap_or(head[..7].to_string()),
            }))
        }
        Err(err) => {
            log::warn!(
                "Couldn't read {}, due to {}",
                revert_head_file.to_string_lossy(),
                err
            );
            Ok(None)
        }
    }
}

// TODO replace with libgit2
fn branch_name_lossy(dir: &Path, hash: &str) -> Res<Option<String>> {
    let out = Command::new("git")
        .args(["for-each-ref", "--format", "%(objectname) %(refname:short)"])
        .current_dir(dir)
        .output()
        .map_err(Error::ReadBranchName)?
        .stdout;

    Ok(String::from_utf8_lossy(&out)
        .lines()
        .find(|line| line.starts_with(hash))
        .map(|line| line.split(' ').nth(1).unwrap().to_string()))
}

pub(crate) fn diff_unstaged(repo: &Repository) -> Res<Diff> {
    let text = String::from_utf8(
        Command::new("git")
            // TODO What if bare repo?
            .current_dir(repo.workdir().expect("Bare repos unhandled"))
            .args(["diff"])
            .output()
            .map_err(Error::GitDiff)?
            .stdout,
    )
    .map_err(Error::GitDiffUtf8)?;

    Ok(Diff {
        file_diffs: gitu_diff::Parser::new(&text).parse_diff().unwrap(),
        text,
    })
}

pub(crate) fn diff_staged(repo: &Repository) -> Res<Diff> {
    let text = String::from_utf8(
        Command::new("git")
            // TODO What if bare repo?
            .current_dir(repo.workdir().expect("Bare repos unhandled"))
            .args(["diff", "--staged"])
            .output()
            .map_err(Error::GitDiff)?
            .stdout,
    )
    .map_err(Error::GitDiffUtf8)?;

    Ok(Diff {
        file_diffs: gitu_diff::Parser::new(&text).parse_diff().unwrap(),
        text,
    })
}

pub(crate) fn show(repo: &Repository, reference: &str) -> Res<Diff> {
    let text = String::from_utf8(
        Command::new("git")
            // TODO What if bare repo?
            .current_dir(repo.workdir().expect("Bare repos unhandled"))
            .args(["show", reference])
            .output()
            .map_err(Error::GitShow)?
            .stdout,
    )
    .map_err(Error::GitShowUtf8)?;

    Ok(Diff {
        file_diffs: gitu_diff::Parser::new(&text).parse_commit().unwrap().diff,
        text,
    })
}

pub(crate) fn show_summary(repo: &Repository, reference: &str) -> Res<Commit> {
    let object = &repo
        .revparse_single(reference)
        .map_err(Error::GitShowMeta)?;
    let commit = object.peel_to_commit().map_err(Error::GitShowMeta)?;

    let author = commit.author();
    let name = author.name().unwrap_or("");
    let email = commit
        .author()
        .email()
        .map(|email| format!("<{}>", email))
        .unwrap_or("".to_string());

    let message = commit
        .message()
        .unwrap_or("")
        .to_string()
        .lines()
        .map(|line| format!("    {}", line))
        .join("\n");

    let offset = chrono::FixedOffset::east_opt(author.when().offset_minutes() * 60).unwrap();
    let time = chrono::DateTime::with_timezone(
        &chrono::DateTime::from_timestamp(author.when().seconds(), 0).unwrap(),
        &offset,
    );

    let details = format!(
        "Author: {}\nDate:   {}\n\n{}",
        [name, &email].join(" "),
        time.to_rfc2822(),
        message
    );

    Ok(Commit {
        hash: commit.id().to_string(),
        details,
    })
}

pub(crate) fn get_current_branch_name(repo: &git2::Repository) -> Res<String> {
    String::from_utf8(
        get_current_branch(repo)?
            .name_bytes()
            .map_err(Error::CurrentBranchName)?
            .to_vec(),
    )
    .map_err(Utf8Error::String)
    .map_err(Error::BranchNameUtf8)
}

pub(crate) fn get_head_name(repo: &git2::Repository) -> Res<String> {
    String::from_utf8(repo.head().map_err(Error::GetHead)?.name_bytes().to_vec())
        .map_err(Utf8Error::String)
        .map_err(Error::BranchNameUtf8)
}

pub(crate) fn get_current_branch(repo: &git2::Repository) -> Res<Branch> {
    let head = repo.head().map_err(Error::GetHead)?;
    if head.is_branch() {
        Ok(Branch::wrap(head))
    } else {
        Err(Error::NotOnBranch)
    }
}
