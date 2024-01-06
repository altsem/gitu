use std::process::Command;

use crate::{diff, process};

pub(crate) fn diff_unstaged() -> String {
    process::pipe(
        process::run("git", &["diff"]).0.as_bytes(),
        "delta",
        &["--color-only"],
    )
}

pub(crate) fn diff_staged() -> String {
    process::pipe(
        process::run("git", &["diff", "--staged"]).0.as_bytes(),
        "delta",
        &["--color-only"],
    )
}

pub(crate) fn log_recent() -> String {
    process::run(
        "git",
        &["log", "-n", "5", "--oneline", "--decorate", "--color"],
    )
    .0
}

pub(crate) fn stage_file(delta: &diff::Delta) {
    process::run("git", &["add", &delta.new_file]);
}

pub(crate) fn stage_hunk(hunk: &diff::Hunk) {
    process::pipe(
        hunk.format_patch().as_bytes(),
        "git",
        &["apply", "--cached"],
    );
}

pub(crate) fn unstage_file(delta: &diff::Delta) {
    process::run("git", &["restore", "--staged", &delta.new_file]);
}

pub(crate) fn unstage_hunk(hunk: &diff::Hunk) {
    process::pipe(
        hunk.format_patch().as_bytes(),
        "git",
        &["apply", "--cached", "--reverse"],
    );
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
