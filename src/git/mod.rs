use git2::{DiffLineType::*, Repository};

use crate::Res;
use std::{
    error::Error,
    fs,
    io::ErrorKind,
    path::Path,
    process::Command,
    str::{self, FromStr},
};

use self::{
    diff::{Delta, Diff, Hunk},
    merge_status::MergeStatus,
    rebase_status::RebaseStatus,
};

pub(crate) mod diff;
pub(crate) mod merge_status;
mod parse;
pub(crate) mod rebase_status;
pub(crate) mod status;

// TODO Check for.git/index.lock and block if it exists
// TODO Use only plumbing commands

pub(crate) fn rebase_status(dir: &Path) -> Res<Option<RebaseStatus>> {
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

pub(crate) fn merge_status(dir: &Path) -> Res<Option<MergeStatus>> {
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

pub(crate) fn diff(dir: &Path, args: &[&str]) -> Res<Diff> {
    // TODO handle args?
    let repo = &Repository::open(dir)?;
    let diff = repo.diff_index_to_workdir(None, None)?;
    convert_diff(diff)
}

// TODO Move elsewhere
pub(crate) fn convert_diff<'a>(diff: git2::Diff) -> Res<Diff> {
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
                    });
                } else {
                    let delta = deltas.last_mut().unwrap();
                    delta.file_header.push_str(line_content);
                }
            }
            Some(hunk) => {
                if is_new_hunk {
                    let delta = deltas.last_mut().unwrap();

                    (*delta).hunks.push(Hunk {
                        file_header: delta.file_header.clone(),
                        old_file: delta.old_file.clone(),
                        new_file: delta.new_file.clone(),
                        old_start: hunk.old_start(),
                        old_lines: hunk.old_lines(),
                        new_start: hunk.new_start(),
                        new_lines: hunk.new_lines(),
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
                            ()
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

pub(crate) fn diff_unstaged(dir: &Path) -> Res<Diff> {
    let repo = &Repository::open(dir)?;
    let diff = repo.diff_index_to_workdir(None, None)?;
    convert_diff(diff)
}

pub(crate) fn diff_staged(dir: &Path) -> Res<Diff> {
    let repo = &Repository::open(dir)?;
    let diff = match repo.head() {
        Ok(head) => repo.diff_tree_to_index(Some(&head.peel_to_tree()?), None, None)?,
        Err(_) => repo.diff_tree_to_index(None, None, None)?,
    };
    convert_diff(diff)
}

pub(crate) fn status(dir: &Path) -> Res<status::Status> {
    run_git(dir, &["status", "--porcelain", "--branch"], &[])
}

pub(crate) fn show(dir: &Path, reference: &str) -> Res<Diff> {
    // TODO Use libigt2
    let repo = Repository::open(dir)?;
    let object = &repo.revparse_single(reference)?;
    let tree = object.peel_to_tree()?;
    let prev = tree.iter().skip(1).next().unwrap();

    let diff = repo.diff_tree_to_tree(
        Some(&prev.to_object(&repo)?.into_tree().unwrap()),
        Some(&object.peel_to_tree()?),
        None,
    )?;
    convert_diff(diff)
}

pub(crate) fn show_summary(dir: &Path, reference: &str) -> Res<String> {
    let repo = Repository::open(dir)?;
    let object = &repo.revparse_single(reference)?;
    let commit = object.peel_to_commit()?;

    Ok(commit.message().unwrap_or("").to_string())
}

// TODO Make this return a more useful type. Vec<Log>?
pub(crate) fn log_recent(dir: &Path) -> Res<String> {
    run_git_no_parse(
        dir,
        &["log", "-n", "5", "--oneline", "--decorate", "--color"],
        &[],
    )
}
// TODO Make this return a more useful type. Vec<Log>?
pub(crate) fn log(dir: &Path, args: &[&str]) -> Res<String> {
    run_git_no_parse(dir, &["log", "--oneline", "--decorate", "--color"], args)
}

// TODO Clean this up
pub(crate) fn show_refs(dir: &Path) -> Res<Vec<(String, String, String)>> {
    let out = Command::new("git")
        .args([
            "for-each-ref",
            "--sort",
            "-creatordate",
            "--format",
            "%(refname:short) %(upstream:short) %(subject)",
            "refs/heads",
        ])
        .current_dir(dir)
        .output()?
        .stdout;

    Ok(str::from_utf8(&out)?
        .lines()
        .map(|line| {
            let mut columns = line.splitn(3, ' ');
            let local = columns.next().unwrap().to_string();
            let remote = columns.next().unwrap().to_string();
            let subject = columns.next().unwrap().to_string();

            (local, remote, subject)
        })
        .collect())
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
pub(crate) fn commit_cmd() -> Command {
    git(&["commit"])
}
pub(crate) fn commit_amend_cmd() -> Command {
    git(&["commit", "--amend"])
}
pub(crate) fn commit_fixup_cmd(reference: &str) -> Command {
    git(&["commit", "--fixup", reference])
}
pub(crate) fn push_cmd() -> Command {
    git(&["push"])
}
pub(crate) fn pull_cmd() -> Command {
    git(&["pull"])
}
pub(crate) fn fetch_all_cmd() -> Command {
    git(&["fetch", "--all"])
}
pub(crate) fn rebase_interactive_cmd(reference: &str) -> Command {
    // TODO autostash flag should be visible as a flag (though set as default)
    git(&["rebase", "-i", "--autostash", reference])
}
pub(crate) fn rebase_autosquash_cmd(reference: &str) -> Command {
    // TODO autostash flag should be visible as a flag (though set as default)
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

pub(crate) fn checkout_file_cmd(file: &str) -> Command {
    git(&["checkout", "--", file])
}

pub(crate) fn checkout_ref_cmd(reference: &str) -> Command {
    git(&["checkout", reference])
}

fn run_git<T: FromStr<Err = Box<dyn Error>>>(
    dir: &Path,
    args: &[&str],
    meta_args: &[&str],
) -> Res<T> {
    let out = Command::new("git")
        .args(&[args, meta_args].concat())
        .current_dir(dir)
        .output()?
        .stdout;

    str::from_utf8(&out)?.parse()
}

fn run_git_no_parse(dir: &Path, args: &[&str], meta_args: &[&str]) -> Res<String> {
    let out = Command::new("git")
        .args(&[args, meta_args].concat())
        .current_dir(dir)
        .output()?
        .stdout;

    Ok(String::from_utf8(out)?)
}

fn git(args: &[&str]) -> Command {
    let mut cmd = Command::new("git");
    cmd.args(args);
    cmd
}
