use git2::{DiffLineType::*, Repository};
use itertools::Itertools;

use self::{
    commit::Commit,
    diff::{Delta, Diff, Hunk},
    merge_status::MergeStatus,
    rebase_status::RebaseStatus,
};
use crate::{git2_opts, Res};
use std::{
    fs,
    io::ErrorKind,
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
            if err.kind() == ErrorKind::NotFound {
                Ok(None)
            } else {
                Err(Box::new(err))
            }
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
            if err.kind() == ErrorKind::NotFound {
                Ok(None)
            } else {
                Err(Box::new(err))
            }
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

// TODO Move elsewhere
pub(crate) fn convert_diff(diff: git2::Diff) -> Res<Diff> {
    let mut deltas = vec![];
    let mut lines = String::new();

    diff.print(git2::DiffFormat::Patch, |delta, maybe_hunk, line| {
        let line_content = str::from_utf8(line.content()).unwrap();
        let is_new_header = line_content.starts_with("diff")
            && line.origin_value() == git2::DiffLineType::FileHeader;
        let is_new_hunk =
            line_content.starts_with("@@") && line.origin_value() == git2::DiffLineType::HunkHeader;

        match maybe_hunk {
            None => {
                if is_new_header {
                    deltas.push(Delta {
                        file_header: line_content.to_string(),
                        old_file: path(&delta.old_file()),
                        new_file: path(&delta.new_file()),
                        hunks: vec![],
                        status: delta.status(),
                    });
                } else {
                    let delta = deltas.last_mut().unwrap();
                    delta.file_header.push_str(line_content);
                }
            }
            Some(hunk) => {
                if is_new_hunk {
                    let delta = deltas.last_mut().unwrap();

                    delta.hunks.push(Hunk {
                        file_header: delta.file_header.clone(),
                        new_file: delta.new_file.clone(),
                        new_start: hunk.new_start(),
                        header: line_content.to_string(),
                        content: String::new(),
                    });
                } else {
                    lines.push_str(line_content);
                    let last_hunk = deltas.last_mut().unwrap().hunks.last_mut().unwrap();

                    match line.origin_value() {
                        Context | Addition | Deletion => {
                            last_hunk
                                .content
                                .push_str(&(format!("{}{}", line.origin(), line_content)));
                        }
                        ContextEOFNL => {
                            // TODO Handle '\ No newline at the end of file'
                        }
                        _ => (),
                    };
                }
            }
        }

        true
    })?;

    Ok(Diff { deltas })
}

// TODO Store PathBuf's instead?
fn path(file: &git2::DiffFile) -> String {
    file.path().unwrap().to_str().unwrap().to_string()
}

pub(crate) fn diff_unstaged(repo: &Repository) -> Res<Diff> {
    let diff = repo.diff_index_to_workdir(None, Some(&mut git2_opts::diff(repo)?))?;
    convert_diff(diff)
}

pub(crate) fn diff_staged(repo: &Repository) -> Res<Diff> {
    let opts = &mut git2_opts::diff(repo)?;

    let diff = match repo.head() {
        Ok(head) => repo.diff_tree_to_index(Some(&head.peel_to_tree()?), None, Some(opts))?,
        Err(_) => repo.diff_tree_to_index(None, None, Some(opts))?,
    };

    convert_diff(diff)
}

pub(crate) fn show(repo: &Repository, reference: &str) -> Res<Diff> {
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
    convert_diff(diff)
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

pub(crate) fn stage_file_cmd(file: &str) -> Command {
    git(&["add", file])
}
pub(crate) fn stage_patch_cmd() -> Command {
    git(&["apply", "--cached"])
}
pub(crate) fn unstage_file_cmd(file: &str) -> Command {
    git(&["restore", "--staged", file])
}
pub(crate) fn unstage_patch_cmd() -> Command {
    git(&["apply", "--cached", "--reverse"])
}
pub(crate) fn discard_unstaged_patch_cmd() -> Command {
    git(&["apply", "--reverse"])
}
pub(crate) fn discard_branch(branch: &str) -> Command {
    git(&["branch", "-d", branch])
}
pub(crate) fn commit_cmd() -> Command {
    git(&["commit"])
}
pub(crate) fn commit_amend_cmd() -> Command {
    git(&["commit", "--amend"])
}
pub(crate) fn commit_fixup_cmd(reference: &str) -> Command {
    git(&["commit", "--fixup", reference])
}
pub(crate) fn fetch_all_cmd() -> Command {
    git(&["fetch", "--all"])
}
pub(crate) fn push_cmd() -> Command {
    git(&["push"])
}
pub(crate) fn pull_cmd() -> Command {
    git(&["pull"])
}
pub(crate) fn rebase_interactive_cmd(reference: &str) -> Command {
    git(&["rebase", "-i", "--autostash", reference])
}
pub(crate) fn rebase_autosquash_cmd(reference: &str) -> Command {
    git(&[
        "rebase",
        "-i",
        "--autosquash",
        "--keep-empty",
        "--autostash",
        reference,
    ])
}
pub(crate) fn rebase_continue_cmd() -> Command {
    git(&["rebase", "--continue"])
}
pub(crate) fn rebase_abort_cmd() -> Command {
    git(&["rebase", "--abort"])
}
pub(crate) fn reset_soft_cmd(reference: &str) -> Command {
    git(&["reset", "--soft", reference])
}
pub(crate) fn reset_mixed_cmd(reference: &str) -> Command {
    git(&["reset", "--mixed", reference])
}
pub(crate) fn reset_hard_cmd(reference: &str) -> Command {
    git(&["reset", "--hard", reference])
}
pub(crate) fn checkout_file_cmd(file: &str) -> Command {
    git(&["checkout", "--", file])
}
pub(crate) fn checkout_new_branch_cmd(name: &str) -> Command {
    git(&["checkout", "-b", name])
}

pub(crate) fn checkout_ref_cmd(reference: &str) -> Command {
    git(&["checkout", reference])
}

fn git(args: &[&str]) -> Command {
    let mut cmd = Command::new("git");
    cmd.args(args);
    cmd
}
