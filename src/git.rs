use crate::{diff::Diff, status::Status, Res};
use std::{process::Command, str};

// TODO Check for.git/index.lock and block if it exists
// TODO Use only plumbing commands

pub(crate) fn status() -> Res<Status> {
    let out = Command::new("git")
        .args(&["status", "--porcelain", "--branch"])
        .output()?
        .stdout;
    Ok(Status::parse(&str::from_utf8(&out)?))
}
pub(crate) fn status_simple() -> Res<String> {
    let out = &Command::new("git")
        .args(&["-c", "color.status=always", "status"])
        .output()?
        .stdout;
    Ok(str::from_utf8(&out)?.replace("[m", "[0m"))
}
pub(crate) fn diff_unstaged() -> Res<Diff> {
    let out = Command::new("git").arg("diff").output()?.stdout;
    Ok(Diff::parse(&str::from_utf8(&out)?))
}
pub(crate) fn show(args: &[&str]) -> Res<Diff> {
    let out = Command::new("git")
        .args(&[&["show"], args].concat())
        .output()?
        .stdout;
    Ok(Diff::parse(str::from_utf8(&out)?))
}
pub(crate) fn show_summary(args: &[&str]) -> Res<String> {
    let out = Command::new("git")
        .args(&[&["show", "--summary", "--decorate", "--color"], args].concat())
        .output()?
        .stdout;
    Ok(str::from_utf8(&out)?.replace("[m", "[0m"))
}
pub(crate) fn diff(args: &[&str]) -> Res<Diff> {
    let out = String::from_utf8(
        Command::new("git")
            .args(&[&["diff"], args].concat())
            .output()?
            .stdout,
    )?;
    Ok(Diff::parse(&out))
}
pub(crate) fn diff_staged() -> Res<Diff> {
    let out = &Command::new("git")
        .args(&["diff", "--staged"])
        .output()?
        .stdout;
    Ok(Diff::parse(str::from_utf8(out)?))
}
// TODO Make this return a more useful type. Vec<Log>?
pub(crate) fn log_recent() -> Res<String> {
    let out = Command::new("git")
        .args(&["log", "-n", "5", "--oneline", "--decorate", "--color"])
        .output()?
        .stdout;
    Ok(String::from_utf8(out)?.replace("[m", "[0m"))
}
// TODO Make this return a more useful type. Vec<Log>?
pub(crate) fn log(args: &[&str]) -> Res<String> {
    let out = Command::new("git")
        .args(&[&["log", "--oneline", "--decorate", "--color"], args].concat())
        .output()?
        .stdout;
    Ok(str::from_utf8(&out)?.replace("[m", "[0m"))
}
pub(crate) fn show_refs() -> Res<Vec<(String, String, String)>> {
    let out = Command::new("git")
        .args(&[
            "for-each-ref",
            "--sort",
            "-creatordate",
            "--format",
            "%(refname) %(upstream) %(subject)",
            "refs/heads",
        ])
        .output()?
        .stdout;

    Ok(str::from_utf8(&out)?
        .lines()
        .map(|line| {
            let mut columns = line.splitn(3, " ");
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
