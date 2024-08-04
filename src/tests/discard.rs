use super::*;

#[test]
pub(crate) fn discard_branch_confirm_prompt() {
    let ctx = TestContext::setup_clone();
    run(ctx.dir.path(), &["git", "branch", "asd"]);
    snapshot!(ctx, "YjK");
}

#[test]
pub(crate) fn discard_branch_yes() {
    let ctx = TestContext::setup_clone();
    run(ctx.dir.path(), &["git", "branch", "asd"]);
    snapshot!(ctx, "YjKy");
}

#[test]
pub(crate) fn discard_branch_no() {
    let ctx = TestContext::setup_clone();
    run(ctx.dir.path(), &["git", "branch", "asd"]);
    snapshot!(ctx, "YjKn");
}

#[test]
pub(crate) fn discard_untracked_file() {
    let ctx = TestContext::setup_clone();
    run(ctx.dir.path(), &["touch", "some-file"]);
    snapshot!(ctx, "jjKy");
}

#[test]
pub(crate) fn discard_untracked_staged_file() {
    let ctx = TestContext::setup_clone();
    run(ctx.dir.path(), &["touch", "some-file"]);
    run(ctx.dir.path(), &["git", "add", "some-file"]);
    snapshot!(ctx, "jsjKy");
}

#[test]
pub(crate) fn discard_file_move() {
    let ctx = TestContext::setup_clone();
    commit(ctx.dir.path(), "new-file", "hello");
    run(ctx.dir.path(), &["git", "mv", "new-file", "moved-file"]);

    snapshot!(ctx, "jjKy");
}

#[test]
pub(crate) fn discard_unstaged_delta() {
    let ctx = TestContext::setup_clone();
    commit(ctx.dir.path(), "file-one", "FOO\nBAR\n");
    fs::write(ctx.dir.child("file-one"), "blahonga\n").unwrap();
    snapshot!(ctx, "jjKy");
}

#[test]
pub(crate) fn discard_unstaged_hunk() {
    let ctx = TestContext::setup_clone();
    commit(ctx.dir.path(), "file-one", "FOO\nBAR\n");
    fs::write(ctx.dir.child("file-one"), "blahonga\n").unwrap();
    snapshot!(ctx, "jj<tab>jKy");
}

#[test]
pub(crate) fn discard_staged_file() {
    let ctx = TestContext::setup_clone();
    commit(ctx.dir.path(), "file-one", "FOO\nBAR\n");
    fs::write(ctx.dir.child("file-one"), "blahonga\n").unwrap();
    run(ctx.dir.path(), &["git", "add", "."]);
    snapshot!(ctx, "jjKy");
}

// FIXME Deleting branches doesn't work with the test-setup
// #[test]
// fn discard_branch() {
//     let mut ctx = TestContext::setup_clone();
//     let mut state = ctx.init_state();
//     state
//         .update(&mut ctx.term, &keys("YjKy"))
//         .unwrap();
//     insta::assert_snapshot!(ctx.redact_buffer());
// }
