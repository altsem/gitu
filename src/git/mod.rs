use crate::Res;
use std::{
    path::Path,
    process::Command,
    str::{self},
};

use self::diff::Diff;

pub(crate) mod diff;
mod parse;
pub(crate) mod status;

// TODO Check for.git/index.lock and block if it exists
// TODO Use only plumbing commands

pub(crate) fn diff(dir: &Path, args: &[&str]) -> Res<Diff> {
    let out = Command::new("git")
        .args(&[&["diff"], args].concat())
        .current_dir(dir)
        .output()?
        .stdout;

    Ok(str::from_utf8(&out)?.parse()?)
}

pub(crate) fn diff_unstaged(dir: &Path) -> Res<Diff> {
    let out = Command::new("git")
        .arg("diff")
        .current_dir(dir)
        .output()?
        .stdout;

    Ok(str::from_utf8(&out)?.parse()?)
}

pub(crate) fn diff_staged(dir: &Path) -> Res<Diff> {
    let out = Command::new("git")
        .args(["diff", "--staged"])
        .current_dir(dir)
        .output()?
        .stdout;

    Ok(str::from_utf8(&out)?.parse()?)
}

pub(crate) fn status(dir: &Path) -> Res<status::Status> {
    let out = Command::new("git")
        .args(["status", "--porcelain", "--branch"])
        .current_dir(dir)
        .output()?
        .stdout;
    Ok(str::from_utf8(&out)?.parse()?)
}
pub(crate) fn status_simple(dir: &Path) -> Res<String> {
    let out = Command::new("git")
        .args(["-c", "color.status=always", "status"])
        .current_dir(dir)
        .output()?
        .stdout;
    Ok(str::from_utf8(&out)?.replace("[m", "[0m"))
}

pub(crate) fn show(dir: &Path, args: &[&str]) -> Res<Diff> {
    let out = Command::new("git")
        .args(&[&["show"], args].concat())
        .current_dir(dir)
        .output()?
        .stdout;
    Ok(str::from_utf8(&out)?.parse()?)
}
pub(crate) fn show_summary(dir: &Path, args: &[&str]) -> Res<String> {
    let out = Command::new("git")
        .args(&[&["show", "--summary", "--decorate", "--color"], args].concat())
        .current_dir(dir)
        .output()?
        .stdout;
    Ok(str::from_utf8(&out)?.replace("[m", "[0m"))
}

// TODO Make this return a more useful type. Vec<Log>?
pub(crate) fn log_recent(dir: &Path) -> Res<String> {
    let out = Command::new("git")
        .args(["log", "-n", "5", "--oneline", "--decorate", "--color"])
        .current_dir(dir)
        .output()?
        .stdout;
    Ok(String::from_utf8(out)?.replace("[m", "[0m"))
}
// TODO Make this return a more useful type. Vec<Log>?
pub(crate) fn log(dir: &Path, args: &[&str]) -> Res<String> {
    let out = Command::new("git")
        .args(&[&["log", "--oneline", "--decorate", "--color"], args].concat())
        .current_dir(dir)
        .output()?
        .stdout;
    Ok(str::from_utf8(&out)?.replace("[m", "[0m"))
}
pub(crate) fn show_refs(dir: &Path) -> Res<Vec<(String, String, String)>> {
    let out = Command::new("git")
        .args([
            "for-each-ref",
            "--sort",
            "-creatordate",
            "--format",
            "%(refname) %(upstream) %(subject)",
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

            (
                local.strip_prefix("refs/heads/").unwrap().to_string(),
                remote
                    .strip_prefix("refs/remotes/")
                    .unwrap_or("")
                    .to_string(),
                subject,
            )
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

fn git(args: &[&str]) -> Command {
    let mut cmd = Command::new("git");
    cmd.args(args);
    cmd
}
