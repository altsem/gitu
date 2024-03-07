use crate::helpers::{clone_and_commit, commit, key, key_code, run, TestContext};
use crossterm::event::KeyCode;
use std::fs;

mod helpers;

#[test]
fn no_repo() {
    let mut ctx = TestContext::setup_init(60, 20);

    ctx.init_state();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn help_menu() {
    let mut ctx = TestContext::setup_init(60, 20);

    let mut state = ctx.init_state();
    gitu::update(&mut state, &mut ctx.term, &[key('h')]).unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn fresh_init() {
    let mut ctx = TestContext::setup_init(60, 20);

    ctx.init_state();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn new_file() {
    let mut ctx = TestContext::setup_init(60, 20);
    run(ctx.dir.path(), &["touch", "new-file"]);

    ctx.init_state();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn unstaged_changes() {
    let mut ctx = TestContext::setup_init(60, 20);
    commit(ctx.dir.path(), "testfile", "testing\ntesttest");
    fs::write(ctx.dir.child("testfile"), "test\ntesttest").expect("error writing to file");

    let mut state = ctx.init_state();
    gitu::update(
        &mut state,
        &mut ctx.term,
        &[key('j'), key('j'), key_code(KeyCode::Tab)],
    )
    .unwrap();

    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn staged_file() {
    let mut ctx = TestContext::setup_init(60, 20);
    run(ctx.dir.path(), &["touch", "new-file"]);
    run(ctx.dir.path(), &["git", "add", "new-file"]);

    ctx.init_state();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn log() {
    let mut ctx = TestContext::setup_clone(60, 20);
    commit(ctx.dir.path(), "firstfile", "testing\ntesttest\n");
    run(ctx.dir.path(), &["git", "tag", "-am", ".", "annotated"]);
    commit(ctx.dir.path(), "secondfile", "testing\ntesttest\n");
    run(ctx.dir.path(), &["git", "tag", "a-tag"]);

    let mut state = ctx.init_state();
    gitu::update(&mut state, &mut ctx.term, &[key('l'), key('l')]).unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn log_other() {
    let mut ctx = TestContext::setup_clone(60, 20);
    commit(ctx.dir.path(), "this-should-be-at-the-top", "");
    commit(ctx.dir.path(), "this-should-not-be-visible", "");

    let mut state = ctx.init_state();
    gitu::update(
        &mut state,
        &mut ctx.term,
        &[key('l'), key('l'), key('j'), key('l'), key('o')],
    )
    .unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn show() {
    let mut ctx = TestContext::setup_clone(60, 20);
    commit(ctx.dir.path(), "firstfile", "This should be visible\n");

    let mut state = ctx.init_state();
    gitu::update(
        &mut state,
        &mut ctx.term,
        &[key('l'), key('l'), key_code(KeyCode::Enter)],
    )
    .unwrap();
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

    ctx.init_state();
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

    ctx.init_state();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn moved_file() {
    let mut ctx = TestContext::setup_clone(60, 20);
    commit(ctx.dir.path(), "new-file", "hello");
    run(ctx.dir.path(), &["git", "mv", "new-file", "moved-file"]);

    ctx.init_state();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn hide_untracked() {
    let mut ctx = TestContext::setup_clone(60, 10);
    run(ctx.dir.path(), &["touch", "i-am-untracked"]);

    let mut state = ctx.init_state();
    let mut config = state.repo.config().unwrap();
    config.set_str("status.showUntrackedFiles", "off").unwrap();
    gitu::update(&mut state, &mut ctx.term, &[key('g')]).unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn new_commit() {
    let mut ctx = TestContext::setup_clone(60, 10);
    commit(ctx.dir.path(), "new-file", "");

    ctx.init_state();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn push() {
    let mut ctx = TestContext::setup_clone(60, 10);
    commit(ctx.dir.path(), "new-file", "");

    let mut state = ctx.init_state();
    gitu::update(&mut state, &mut ctx.term, &[key('P'), key('p')]).unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn fetch_all() {
    let mut ctx = TestContext::setup_clone(60, 10);
    clone_and_commit(&ctx.remote_dir, "remote-file", "hello");

    let mut state = ctx.init_state();
    gitu::update(&mut state, &mut ctx.term, &[key('f'), key('a')]).unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn pull() {
    let mut ctx = TestContext::setup_clone(60, 10);
    clone_and_commit(&ctx.remote_dir, "remote-file", "hello");

    let mut state = ctx.init_state();
    gitu::update(&mut state, &mut ctx.term, &[key('F'), key('p')]).unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn discard_branch_confirm() {
    let mut ctx = TestContext::setup_clone(60, 10);

    let mut state = ctx.init_state();
    gitu::update(&mut state, &mut ctx.term, &[key('y'), key('j'), key('K')]).unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn discard_branch() {
    let mut ctx = TestContext::setup_clone(60, 10);

    let mut state = ctx.init_state();
    gitu::update(
        &mut state,
        &mut ctx.term,
        &[key('y'), key('j'), key('K'), key('y')],
    )
    .unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn reset_menu() {
    let mut ctx = TestContext::setup_clone(60, 10);
    commit(ctx.dir.path(), "unwanted-file", "");

    let mut state = ctx.init_state();
    gitu::update(
        &mut state,
        &mut ctx.term,
        &[key('l'), key('l'), key('j'), key('x')],
    )
    .unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn reset_soft() {
    let mut ctx = TestContext::setup_clone(60, 10);
    commit(ctx.dir.path(), "unwanted-file", "");

    let mut state = ctx.init_state();
    gitu::update(
        &mut state,
        &mut ctx.term,
        &[key('l'), key('l'), key('j'), key('x'), key('s'), key('q')],
    )
    .unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn reset_mixed() {
    let mut ctx = TestContext::setup_clone(60, 10);
    commit(ctx.dir.path(), "unwanted-file", "");

    let mut state = ctx.init_state();
    gitu::update(
        &mut state,
        &mut ctx.term,
        &[key('l'), key('l'), key('j'), key('x'), key('m'), key('q')],
    )
    .unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn reset_hard() {
    let mut ctx = TestContext::setup_clone(60, 10);
    commit(ctx.dir.path(), "unwanted-file", "");

    let mut state = ctx.init_state();
    gitu::update(
        &mut state,
        &mut ctx.term,
        &[key('l'), key('l'), key('j'), key('x'), key('h'), key('q')],
    )
    .unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn show_refs() {
    let mut ctx = TestContext::setup_clone(60, 10);
    run(ctx.dir.path(), &["git", "tag", "same-name"]);
    run(ctx.dir.path(), &["git", "checkout", "-b", "same-name"]);

    let mut state = ctx.init_state();
    gitu::update(&mut state, &mut ctx.term, &[key('y')]).unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn checkout_new_branch() {
    let mut ctx = TestContext::setup_clone(60, 10);

    let mut state = ctx.init_state();
    gitu::update(
        &mut state,
        &mut ctx.term,
        &[
            key('b'),
            key('c'),
            key('f'),
            // Don't want to create branch 'f', try again
            key_code(KeyCode::Esc),
            key('b'),
            key('c'),
            key('x'),
            key_code(KeyCode::Enter),
        ],
    )
    .unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
}
