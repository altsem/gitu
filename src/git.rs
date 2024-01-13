use crate::{diff, process};
use std::process::Command;

const COLOR_CMD: &[&str] = &["delta", "--color-only"];

pub(crate) fn diff_unstaged() -> String {
    process::pipe(process::run(&["git", "diff"]).0.as_bytes(), COLOR_CMD).0
}

pub(crate) fn show(reference: &str) -> String {
    process::pipe(
        process::run(&["git", "show", reference]).0.as_bytes(),
        COLOR_CMD,
    )
    .0
}

pub(crate) fn show_summary(reference: &str) -> String {
    process::pipe(
        process::run(&["git", "show", "--summary", reference])
            .0
            .as_bytes(),
        COLOR_CMD,
    )
    .0
}

pub(crate) fn diff_staged() -> String {
    process::pipe(
        process::run(&["git", "diff", "--staged"]).0.as_bytes(),
        COLOR_CMD,
    )
    .0
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

pub(crate) fn log() -> String {
    process::run(&["git", "log", "--oneline", "--decorate", "--color"]).0
}

pub(crate) fn stage_file_cmd(delta: &diff::Delta) -> Command {
    let mut cmd = Command::new("git");
    cmd.args(&["add", &delta.new_file]);
    cmd
}

pub(crate) fn stage_patch_cmd() -> Command {
    let mut cmd = Command::new("git");
    cmd.args(&["apply", "--cached"]);
    cmd
}

pub(crate) fn unstage_file_cmd(delta: &diff::Delta) -> Command {
    let mut cmd = Command::new("git");
    cmd.args(&["restore", "--staged", &delta.new_file]);
    cmd
}

pub(crate) fn unstage_patch_cmd() -> Command {
    let mut cmd = Command::new("git");
    cmd.args(&["apply", "--cached", "--reverse"]);
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
    cmd.args(&["fetch", "--all"]);
    cmd
}
