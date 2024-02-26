use crate::helpers::{clone_and_commit, commit, key, key_code, run, TestContext};
use crossterm::event::KeyCode;
use std::fs;

mod helpers;

#[test]
fn no_repo() {
    let ctx = TestContext::setup_init(60, 20);
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn help_menu() {
    let mut ctx = TestContext::setup_init(60, 20);
    ctx.update(&[key('h')]);
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn fresh_init() {
    let mut ctx = TestContext::setup_init(60, 20);
    ctx.update(&[key('g')]);
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn new_file() {
    let mut ctx = TestContext::setup_init(60, 20);
    run(ctx.dir.path(), &["touch", "new-file"]);
    ctx.update(&[key('g')]);
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn unstaged_changes() {
    let mut ctx = TestContext::setup_init(60, 20);
    commit(ctx.dir.path(), "testfile", "testing\ntesttest");
    fs::write(ctx.dir.child("testfile"), "test\ntesttest").expect("error writing to file");

    ctx.update(&[key('g')]);
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn staged_file() {
    let mut ctx = TestContext::setup_init(60, 20);
    run(ctx.dir.path(), &["touch", "new-file"]);
    run(ctx.dir.path(), &["git", "add", "new-file"]);
    ctx.update(&[key('g')]);
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn log() {
    let mut ctx = TestContext::setup_clone(60, 20);
    commit(ctx.dir.path(), "firstfile", "testing\ntesttest\n");
    commit(ctx.dir.path(), "secondfile", "testing\ntesttest\n");
    ctx.update(&[key('g'), key('l'), key('l')]);
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn log_other() {
    let mut ctx = TestContext::setup_clone(60, 20);
    commit(ctx.dir.path(), "this-should-be-at-the-top", "");
    commit(ctx.dir.path(), "this-should-not-be-visible", "");

    ctx.update(&[key('g'), key('l'), key('l'), key('j'), key('l'), key('o')]);
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn show() {
    let mut ctx = TestContext::setup_clone(60, 20);
    commit(ctx.dir.path(), "firstfile", "This should be visible\n");
    ctx.update(&[key('g'), key('l'), key('l'), key_code(KeyCode::Enter)]);
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn rebase_conflict() {
    let mut ctx = TestContext::setup_clone(60, 20);
    commit(ctx.dir.path(), "new-file", "hello");

    run(ctx.dir.path(), &["git", "checkout", "-b", "other-branch"]);
    commit(ctx.dir.path(), "new-file", "hey");

    run(ctx.dir.path(), &["git", "checkout", "main"]);
    commit(ctx.dir.path(), "new-file", "hi");

    run(ctx.dir.path(), &["git", "checkout", "other-branch"]);
    run(ctx.dir.path(), &["git", "rebase", "main"]);

    ctx.update(&[key('g')]);
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn merge_conflict() {
    let mut ctx = TestContext::setup_clone(60, 20);
    commit(ctx.dir.path(), "new-file", "hello");

    run(ctx.dir.path(), &["git", "checkout", "-b", "other-branch"]);
    commit(ctx.dir.path(), "new-file", "hey");

    run(ctx.dir.path(), &["git", "checkout", "main"]);
    commit(ctx.dir.path(), "new-file", "hi");

    run(ctx.dir.path(), &["git", "merge", "other-branch"]);

    ctx.update(&[key('g')]);
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn moved_file() {
    let mut ctx = TestContext::setup_clone(60, 20);
    commit(ctx.dir.path(), "new-file", "hello");
    run(ctx.dir.path(), &["git", "mv", "new-file", "moved-file"]);

    ctx.update(&[key('g')]);
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn hide_untracked() {
    let mut ctx = TestContext::setup_clone(60, 10);
    let mut config = ctx.state.repo.config().unwrap();
    config.set_str("status.showUntrackedFiles", "off").unwrap();
    run(ctx.dir.path(), &["touch", "i-am-untracked"]);

    ctx.update(&[key('g')]);
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn new_commit() {
    let mut ctx = TestContext::setup_clone(60, 10);
    commit(ctx.dir.path(), "new-file", "");

    ctx.update(&[key('g')]);
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn push() {
    let mut ctx = TestContext::setup_clone(60, 10);
    commit(ctx.dir.path(), "new-file", "");

    ctx.update(&[key('P'), key('p')]);
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn fetch_all() {
    let mut ctx = TestContext::setup_clone(60, 10);
    clone_and_commit(&ctx.remote_dir, "remote-file", "hello");

    ctx.update(&[key('f'), key('a')]);
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn pull() {
    let mut ctx = TestContext::setup_clone(60, 10);
    clone_and_commit(&ctx.remote_dir, "remote-file", "hello");

    ctx.update(&[key('F'), key('p')]);
    insta::assert_snapshot!(ctx.redact_buffer());
}
