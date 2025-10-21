use diff::Diff;
use git2::{Branch, DiffFormat, Repository};
use itertools::Itertools;
use remote::get_branch_upstream;

use self::{commit::Commit, merge_status::MergeStatus, rebase_status::RebaseStatus};
use crate::{
    Res,
    error::{Error, Utf8Error},
    gitu_diff,
};
use std::{
    fs,
    path::Path,
    process::Command,
    str::{self, FromStr},
};

pub(crate) mod commit;
pub(crate) mod diff;
pub(crate) mod merge_status;
mod parse;
pub(crate) mod rebase_status;
pub(crate) mod remote;
pub(crate) mod status;

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

#[derive(Debug, Clone)]
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
    let diff = repo
        .diff_index_to_workdir(None, None)
        .map_err(Error::GitDiff)?;

    let mut text = Vec::new();
    diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
        let prefix: &[u8] = match line.origin() {
            ' ' => b" ",
            '+' => b"+",
            '-' => b"-",
            _ => b"",
        };
        text.extend_from_slice(prefix);
        text.extend_from_slice(line.content());
        true
    })
    .map_err(Error::GitDiff)?;

    let mut text = String::from_utf8(text).map_err(Error::GitDiffUtf8)?;
    if !text.ends_with('\n') {
        text.push('\n');
    }

    let file_diffs = gitu_diff::Parser::new(&text).parse_diff().unwrap();

    Ok(Diff { file_diffs, text })
}

pub(crate) fn diff_staged(repo: &Repository) -> Res<Diff> {
    let head_tree = match repo.head() {
        Ok(head) => Some(head.peel_to_tree().map_err(Error::GitDiff)?),
        Err(_) => None, // No HEAD yet
    };
    let diff = repo
        .diff_tree_to_index(head_tree.as_ref(), None, None)
        .map_err(Error::GitDiff)?;

    let mut text = Vec::new();
    diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
        let prefix: &[u8] = match line.origin() {
            ' ' => b" ",
            '+' => b"+",
            '-' => b"-",
            _ => b"",
        };
        text.extend_from_slice(prefix);
        text.extend_from_slice(line.content());
        true
    })
    .map_err(Error::GitDiff)?;

    let mut text = String::from_utf8(text).map_err(Error::GitDiffUtf8)?;
    if !text.ends_with('\n') {
        text.push('\n');
    }

    let file_diffs = gitu_diff::Parser::new(&text).parse_diff().unwrap();

    Ok(Diff { file_diffs, text })
}

pub(crate) fn status(dir: &Path) -> Res<status::Status> {
    let text = String::from_utf8(
        Command::new("git")
            .current_dir(dir)
            .args(["status", "--porcelain", "--branch"])
            .output()
            .unwrap()
            .stdout,
    )
    .map_err(Error::GitDiffUtf8)?;

    Ok(status::Status::from_str(&text).unwrap())
}

pub(crate) fn show(repo: &Repository, reference: &str) -> Res<Diff> {
    let obj = repo.revparse_single(reference).map_err(Error::GitShow)?;
    let commit = obj.peel_to_commit().map_err(Error::GitShow)?;

    let parent = if commit.parent_count() > 0 {
        Some(commit.parent(0).map_err(Error::GitShow)?)
    } else {
        None
    };

    let diff = if let Some(parent) = parent {
        repo.diff_tree_to_tree(
            Some(&parent.tree().map_err(Error::GitShow)?),
            Some(&commit.tree().map_err(Error::GitShow)?),
            None,
        )
    } else {
        repo.diff_tree_to_tree(None, Some(&commit.tree().map_err(Error::GitShow)?), None)
    }
    .map_err(Error::GitShow)?;

    let mut text = Vec::new();
    diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
        text.extend_from_slice(line.content());
        true
    })
    .map_err(Error::GitShow)?;

    let text = String::from_utf8(text).map_err(Error::GitShowUtf8)?;

    Ok(Diff {
        file_diffs: gitu_diff::Parser::new(&text).parse_diff().unwrap(),
        text,
    })
}

pub(crate) fn stash_show(repo: &Repository, stash_ref: &str) -> Res<Diff> {
    use git2::DiffFormat;

    let obj = repo.revparse_single(stash_ref).map_err(Error::GitShow)?;
    let commit = obj.peel_to_commit().map_err(Error::GitShow)?;

    let parent = commit.parent(0).map_err(Error::GitShow)?;
    let diff = repo
        .diff_tree_to_tree(
            Some(&parent.tree().map_err(Error::GitShow)?),
            Some(&commit.tree().map_err(Error::GitShow)?),
            None,
        )
        .map_err(Error::GitShow)?;

    let mut text = Vec::new();
    diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
        text.extend_from_slice(line.content());
        true
    })
    .map_err(Error::GitShow)?;

    let text = String::from_utf8(text).map_err(Error::GitShowUtf8)?;

    Ok(Diff {
        file_diffs: gitu_diff::Parser::new(&text).parse_diff().unwrap(),
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
        .map(|email| format!("<{email}>"))
        .unwrap_or("".to_string());

    let message = commit
        .message()
        .unwrap_or("")
        .to_string()
        .lines()
        .map(|line| format!("    {line}"))
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

pub(crate) fn get_current_branch(repo: &git2::Repository) -> Res<Branch<'_>> {
    let head = repo.head().map_err(Error::GetHead)?;
    if head.is_branch() {
        Ok(Branch::wrap(head))
    } else {
        Err(Error::NotOnBranch)
    }
}

pub(crate) fn is_branch_merged(repo: &git2::Repository, name: &str) -> Res<bool> {
    let branch = repo
        .find_branch(name, git2::BranchType::Local)
        .map_err(Error::IsBranchMerged)?;

    let upstream = get_branch_upstream(&branch)?;

    let reference = match upstream {
        Some(u) => u.into_reference(),
        None => repo.head().map_err(Error::GetHead)?,
    };

    let ref_commit = reference.peel_to_commit().map_err(Error::IsBranchMerged)?;

    let commit = branch
        .into_reference()
        .peel_to_commit()
        .map_err(Error::IsBranchMerged)?;

    Ok(commit.id() == ref_commit.id()
        || repo
            .graph_descendant_of(ref_commit.id(), commit.id())
            .map_err(Error::IsBranchMerged)?)
}
