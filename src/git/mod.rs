use git2::Repository;
use itertools::Itertools;

use self::{commit::Commit, diff::Diff, merge_status::MergeStatus, rebase_status::RebaseStatus};
use crate::{config::Config, git2_opts, Res};
use std::{
    ffi::OsStr,
    fs,
    path::Path,
    process::Command,
    str::{self},
};

pub(crate) mod commit;
pub(crate) mod diff;
pub(crate) mod merge_status;
pub(crate) mod rebase_status;

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
                onto: branch_name(dir, &onto_hash)?.unwrap_or_else(|| onto_hash[..7].to_string()),
                head_name: fs::read_to_string(rebase_head_name_file)?
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
                head: branch_name(dir, &head)?.unwrap_or(head[..7].to_string()),
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

// TODO replace with libgit2
fn branch_name(dir: &Path, hash: &str) -> Res<Option<String>> {
    let out = Command::new("git")
        .args(["for-each-ref", "--format", "%(objectname) %(refname:short)"])
        .current_dir(dir)
        .output()?
        .stdout;

    Ok(str::from_utf8(&out)?
        .lines()
        .find(|line| line.starts_with(hash))
        .map(|line| line.split(' ').nth(1).unwrap().to_string()))
}

pub(crate) fn diff_unstaged(config: &Config, repo: &Repository) -> Res<Diff> {
    let diff = repo.diff_index_to_workdir(None, Some(&mut git2_opts::diff(repo)?))?;
    diff::convert_diff(config, repo, diff, true)
}

pub(crate) fn diff_staged(config: &Config, repo: &Repository) -> Res<Diff> {
    let opts = &mut git2_opts::diff(repo)?;

    let diff = match repo.head() {
        Ok(head) => repo.diff_tree_to_index(Some(&head.peel_to_tree()?), None, Some(opts))?,
        Err(_) => repo.diff_tree_to_index(None, None, Some(opts))?,
    };

    diff::convert_diff(config, repo, diff, false)
}

pub(crate) fn show(config: &Config, repo: &Repository, reference: &str) -> Res<Diff> {
    let object = &repo.revparse_single(reference)?;

    let commit = object.peel_to_commit()?;
    let tree = commit.tree()?;
    let parent_tree = commit
        .parents()
        .next()
        .and_then(|parent| parent.tree().ok());

    let diff = repo.diff_tree_to_tree(
        parent_tree.as_ref(),
        Some(&tree),
        Some(&mut git2_opts::diff(repo)?),
    )?;

    diff::convert_diff(config, repo, diff, false)
}

pub(crate) fn show_summary(repo: &Repository, reference: &str) -> Res<Commit> {
    let object = &repo.revparse_single(reference)?;
    let commit = object.peel_to_commit()?;

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

pub(crate) fn stage_file_cmd(file: &OsStr) -> Command {
    git([OsStr::new("add"), file])
}
pub(crate) fn stage_patch_cmd() -> Command {
    git(["apply", "--cached"])
}
pub(crate) fn stage_line_cmd() -> Command {
    git(["apply", "--cached", "--recount"])
}
pub(crate) fn unstage_file_cmd(file: &OsStr) -> Command {
    git([OsStr::new("restore"), OsStr::new("--staged"), file])
}
pub(crate) fn unstage_patch_cmd() -> Command {
    git(["apply", "--cached", "--reverse"])
}
pub(crate) fn unstage_line_cmd() -> Command {
    git(["apply", "--cached", "--reverse", "--recount"])
}
pub(crate) fn discard_unstaged_patch_cmd() -> Command {
    git(["apply", "--reverse"])
}
pub(crate) fn discard_branch(branch: &OsStr) -> Command {
    git([OsStr::new("branch"), OsStr::new("-d"), branch])
}
pub(crate) fn commit_fixup_cmd(reference: &OsStr) -> Command {
    git([OsStr::new("commit"), OsStr::new("--fixup"), reference])
}
pub(crate) fn reset_soft_cmd(reference: &OsStr) -> Command {
    git([OsStr::new("reset"), OsStr::new("--soft"), reference])
}
pub(crate) fn reset_mixed_cmd(reference: &OsStr) -> Command {
    git([OsStr::new("reset"), OsStr::new("--mixed"), reference])
}
pub(crate) fn reset_hard_cmd(reference: &OsStr) -> Command {
    git([OsStr::new("reset"), OsStr::new("--hard"), reference])
}
pub(crate) fn checkout_file_cmd(file: &OsStr) -> Command {
    git([
        OsStr::new("checkout"),
        OsStr::new("HEAD"),
        OsStr::new("--"),
        file,
    ])
}

pub(crate) fn git<I, S>(args: I) -> Command
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut cmd = Command::new("git");
    cmd.args(args);
    cmd
}
