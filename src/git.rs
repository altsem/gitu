use crate::{
    diff::{self, Diff},
    process,
    status::Status,
};
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
    let mut cmd = Command::new("git");
    cmd.args(["add", &file]);
    cmd
}

pub(crate) fn stage_patch_cmd() -> Command {
    let mut cmd = Command::new("git");
    cmd.args(["apply", "--cached"]);
    cmd
}

pub(crate) fn unstage_file_cmd(delta: &diff::Delta) -> Command {
    let mut cmd = Command::new("git");
    cmd.args(["restore", "--staged", &delta.new_file]);
    cmd
}

pub(crate) fn unstage_patch_cmd() -> Command {
    let mut cmd = Command::new("git");
    cmd.args(["apply", "--cached", "--reverse"]);
    cmd
}

pub(crate) fn commit_cmd() -> Command {
    let mut cmd = Command::new("git");
    cmd.arg("commit");
    cmd
}

pub(crate) fn push_cmd() -> Command {
    let mut cmd = Command::new("git");
    cmd.arg("push");
    cmd
}

pub(crate) fn pull_cmd() -> Command {
    let mut cmd = Command::new("git");
    cmd.arg("pull");
    cmd
}

pub(crate) fn fetch_all_cmd() -> Command {
    let mut cmd = Command::new("git");
    cmd.args(["fetch", "--all"]);
    cmd
}

pub(crate) fn rebase_interactive_cmd(reference: &str) -> Command {
    let mut cmd = Command::new("git");
    // TODO autostash flag should be visible as a flag (though set as default)
    cmd.args(["rebase", "--autostash", "-i", reference]);
    cmd
}
