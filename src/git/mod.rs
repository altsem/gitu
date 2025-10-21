use diff::Diff;
use git2::{Branch, DiffFormat, Repository, RepositoryState};
use itertools::Itertools;
use remote::get_branch_upstream;

use self::{commit::Commit, merge_status::MergeStatus, rebase_status::RebaseStatus};
use crate::{
    Res,
    error::{Error, Utf8Error},
    gitu_diff,
};
use std::{fs, path::Path, process::Command, str};

pub(crate) mod commit;
pub(crate) mod diff;
pub(crate) mod merge_status;
pub(crate) mod rebase_status;
pub(crate) mod remote;
pub(crate) mod status;

pub(crate) fn rebase_status(repo: &Repository) -> Res<Option<RebaseStatus>> {
    if !matches!(
        repo.state(),
        RepositoryState::Rebase | RepositoryState::RebaseMerge | RepositoryState::RebaseInteractive
    ) {
        return Ok(None);
    }

    let dir = repo.workdir().expect("No workdir");
    let mut onto_file = dir.to_path_buf();
    onto_file.push(".git/rebase-merge/onto");

    let onto_content = fs::read_to_string(&onto_file).map_err(Error::ReadRebaseStatusFile)?;
    let onto_oid = git2::Oid::from_str(onto_content.trim()).map_err(|_| {
        Error::ReadRebaseStatusFile(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Invalid OID",
        ))
    })?;

    let onto = branch_name_lossy(dir, &onto_oid.to_string())?
        .unwrap_or_else(|| onto_oid.to_string()[..7].to_string());

    let head_name = if let Ok(rebase) = repo.open_rebase(None) {
        rebase
            .orig_head_name()
            .map(|s| s.strip_prefix("refs/heads/").unwrap_or(s).to_string())
            .unwrap_or_else(|| "HEAD".to_string())
    } else {
        let mut head_name_file = dir.to_path_buf();
        head_name_file.push(".git/rebase-merge/head-name");
        fs::read_to_string(&head_name_file)
            .map(|s| {
                s.trim()
                    .strip_prefix("refs/heads/")
                    .unwrap_or(s.trim())
                    .to_string()
            })
            .unwrap_or_else(|_| "HEAD".to_string())
    };

    Ok(Some(RebaseStatus { onto, head_name }))
}

pub(crate) fn merge_status(repo: &Repository) -> Res<Option<MergeStatus>> {
    if repo.state() != RepositoryState::Merge {
        return Ok(None);
    }

    let dir = repo.workdir().expect("No workdir");
    let mut merge_head_file = dir.to_path_buf();
    merge_head_file.push(".git/MERGE_HEAD");

    let content = fs::read_to_string(&merge_head_file).map_err(Error::ReadRebaseStatusFile)?;
    let head_oid = git2::Oid::from_str(content.trim()).map_err(|_| {
        Error::ReadRebaseStatusFile(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Invalid OID",
        ))
    })?;

    let head = branch_name_lossy(dir, &head_oid.to_string())?
        .unwrap_or_else(|| head_oid.to_string()[..7].to_string());

    Ok(Some(MergeStatus { head }))
}

#[derive(Debug, Clone)]
pub(crate) struct RevertStatus {
    pub head: String,
}

pub(crate) fn revert_status(repo: &Repository) -> Res<Option<RevertStatus>> {
    if repo.state() != RepositoryState::Revert {
        return Ok(None);
    }

    let dir = repo.workdir().expect("No workdir");
    let mut revert_head_file = dir.to_path_buf();
    revert_head_file.push(".git/REVERT_HEAD");

    let content = fs::read_to_string(&revert_head_file).map_err(Error::ReadRebaseStatusFile)?;
    let head_oid = git2::Oid::from_str(content.trim()).map_err(|_| {
        Error::ReadRebaseStatusFile(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Invalid OID",
        ))
    })?;

    let head = branch_name_lossy(dir, &head_oid.to_string())?
        .unwrap_or_else(|| head_oid.to_string()[..7].to_string());

    Ok(Some(RevertStatus { head }))
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
        .map_err(|e| Error::git_operation("diff unstaged", e))?;

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

pub(crate) fn status(repo: &Repository) -> Res<status::Status> {
    let branch_status = get_branch_status(repo)?;
    let files = get_status_files(repo)?;

    Ok(status::Status {
        branch_status,
        files,
    })
}

fn get_branch_status(repo: &Repository) -> Res<status::BranchStatus> {
    let head = match repo.head() {
        Ok(head) if head.is_branch() => Branch::wrap(head),
        _ => {
            return Ok(status::BranchStatus {
                local: None,
                remote: None,
                ahead: 0,
                behind: 0,
            });
        }
    };

    let local = head.name().ok().flatten().map(|n| n.to_string());
    let remote = if let Ok(upstream) = head.upstream() {
        upstream.name().ok().flatten().map(|n| n.to_string())
    } else {
        None
    };

    let (ahead, behind) = if let (Ok(_upstream), Ok(head_commit), Ok(upstream_commit)) = (
        head.upstream(),
        head.get().peel_to_commit(),
        head.upstream().and_then(|u| u.get().peel_to_commit()),
    ) {
        let (ahead_usize, behind_usize) = repo
            .graph_ahead_behind(head_commit.id(), upstream_commit.id())
            .unwrap_or((0, 0));
        (ahead_usize as u32, behind_usize as u32)
    } else {
        (0, 0)
    };

    Ok(status::BranchStatus {
        local,
        remote,
        ahead,
        behind,
    })
}

fn get_status_files(repo: &Repository) -> Res<Vec<status::StatusFile>> {
    let mut files = Vec::new();
    let statuses = repo
        .statuses(None)
        .map_err(|e| Error::git_operation("status", e))?;
    for entry in statuses.iter() {
        let status = entry.status();
        let path = entry.path().unwrap_or("").to_string();
        let new_path = entry
            .index_to_workdir()
            .and_then(|diff| diff.new_file().path())
            .map(|p| p.to_string_lossy().to_string());

        let status_code = [
            status_char(
                status,
                git2::Status::INDEX_NEW
                    | git2::Status::INDEX_MODIFIED
                    | git2::Status::INDEX_DELETED
                    | git2::Status::INDEX_RENAMED
                    | git2::Status::INDEX_TYPECHANGE,
            ),
            status_char(
                status,
                git2::Status::WT_NEW
                    | git2::Status::WT_MODIFIED
                    | git2::Status::WT_DELETED
                    | git2::Status::WT_RENAMED
                    | git2::Status::WT_TYPECHANGE,
            ),
        ];

        files.push(status::StatusFile {
            status_code,
            path,
            new_path,
        });
    }
    Ok(files)
}

fn status_char(status: git2::Status, mask: git2::Status) -> char {
    if status.intersects(mask) {
        if mask.contains(git2::Status::INDEX_NEW) || mask.contains(git2::Status::WT_NEW) {
            'A'
        } else if mask.contains(git2::Status::INDEX_MODIFIED)
            || mask.contains(git2::Status::WT_MODIFIED)
        {
            'M'
        } else if mask.contains(git2::Status::INDEX_DELETED)
            || mask.contains(git2::Status::WT_DELETED)
        {
            'D'
        } else if mask.contains(git2::Status::INDEX_RENAMED)
            || mask.contains(git2::Status::WT_RENAMED)
        {
            'R'
        } else if mask.contains(git2::Status::INDEX_TYPECHANGE)
            || mask.contains(git2::Status::WT_TYPECHANGE)
        {
            'T'
        } else {
            '?'
        }
    } else if status.is_empty() {
        ' '
    } else {
        '?'
    }
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
