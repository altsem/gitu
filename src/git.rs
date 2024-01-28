use crate::{diff::Diff, process, status::Status};
use std::process::Command;

// TODO Check for.git/index.lock and block if it exists
// TODO Use only plumbing commands

pub(crate) fn status() -> Status {
    Status::parse(&process::run(&["git", "status", "--porcelain", "--branch"]).0)
}
pub(crate) fn status_simple() -> String {
    process::run(&["git", "status"]).0
}
pub(crate) fn diff_unstaged() -> Diff {
    Diff::parse(&process::run(&["git", "diff"]).0)
}
pub(crate) fn show(args: &[&str]) -> Diff {
    Diff::parse(&process::run(&[&["git", "show"], args].concat()).0)
}
pub(crate) fn show_summary(args: &[&str]) -> String {
    process::run(&[&["git", "show", "--summary", "--decorate", "--color"], args].concat()).0
}
pub(crate) fn diff(args: &[&str]) -> Diff {
    Diff::parse(&process::run(&[&["git", "diff"], args].concat()).0)
}
pub(crate) fn diff_staged() -> Diff {
    Diff::parse(&process::run(&["git", "diff", "--staged"]).0)
}
pub(crate) fn log_recent() -> String {
    process::run(&[
        "git",
        "log",
        "-n",
        "5",
        "--oneline",
        "--decorate",
        "--color",
    ])
    .0
}

pub(crate) fn log(args: &[&str]) -> String {
    process::run(&[&["git", "log", "--oneline", "--decorate", "--color"], args].concat()).0
}

pub(crate) fn stage_file_cmd(file: &str) -> Command {
    git(&["add", &file])
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

fn git(args: &[&str]) -> Command {
    let mut cmd = Command::new("git");
    cmd.args(args);
    cmd
}
