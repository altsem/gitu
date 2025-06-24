use super::*;

fn setup() -> TestContext {
    let ctx = TestContext::setup_clone();
    run(ctx.dir.path(), &["git", "checkout", "-b", "merged"]);
    run(ctx.dir.path(), &["git", "checkout", "-b", "unmerged"]);
    commit(ctx.dir.path(), "first commit", "");
    run(ctx.dir.path(), &["git", "checkout", "main"]);
    ctx
}

#[test]
pub(crate) fn discard_branch_confirm_prompt() {
    let ctx = TestContext::setup_clone();
    run(ctx.dir.path(), &["git", "branch", "asd"]);
    snapshot!(ctx, "<shift+Y>j<shift+K>");
}

#[test]
pub(crate) fn discard_branch_yes() {
    let ctx = TestContext::setup_clone();
    run(ctx.dir.path(), &["git", "branch", "asd"]);
    snapshot!(ctx, "<shift+Y>j<shift+K>y");
}

#[test]
pub(crate) fn discard_branch_no() {
    let ctx = TestContext::setup_clone();
    run(ctx.dir.path(), &["git", "branch", "asd"]);
    snapshot!(ctx, "<shift+Y>j<shift+K>n");
}

#[test]
pub(crate) fn discard_untracked_file() {
    let ctx = TestContext::setup_clone();
    run(ctx.dir.path(), &["touch", "some-file"]);
    snapshot!(ctx, "jj<shift+K>y");
}

#[test]
pub(crate) fn discard_untracked_staged_file() {
    let ctx = TestContext::setup_clone();
    run(ctx.dir.path(), &["touch", "some-file"]);
    run(ctx.dir.path(), &["git", "add", "some-file"]);
    snapshot!(ctx, "jsj<shift+K>y");
}

#[test]
pub(crate) fn discard_file_move() {
    let ctx = TestContext::setup_clone();
    commit(ctx.dir.path(), "new-file", "hello");
    run(ctx.dir.path(), &["git", "mv", "new-file", "moved-file"]);

    snapshot!(ctx, "jj<shift+K>y");
}

#[test]
pub(crate) fn discard_unstaged_delta() {
    let ctx = TestContext::setup_clone();
    commit(ctx.dir.path(), "file-one", "FOO\nBAR\n");
    fs::write(ctx.dir.child("file-one"), "blahonga\n").unwrap();
    snapshot!(ctx, "jj<shift+K>y");
}

#[test]
pub(crate) fn discard_unstaged_hunk() {
    let ctx = TestContext::setup_clone();
    commit(ctx.dir.path(), "file-one", "FOO\nBAR\n");
    fs::write(ctx.dir.child("file-one"), "blahonga\n").unwrap();
    snapshot!(ctx, "jj<tab>j<shift+K>y");
}

#[test]
pub(crate) fn discard_unstaged_line() {
    let ctx = TestContext::setup_clone();
    commit(ctx.dir.path(), "file-one", "FOO\nBAR\n");
    fs::write(ctx.dir.child("file-one"), "blahonga\n").unwrap();
    snapshot!(ctx, "jj<tab>j<ctrl+j><shift+K>y<ctrl+j><ctrl+j><shift+K>y");
}

#[test]
pub(crate) fn discard_staged_file() {
    let ctx = TestContext::setup_clone();
    commit(ctx.dir.path(), "file-one", "FOO\nBAR\n");
    fs::write(ctx.dir.child("file-one"), "blahonga\n").unwrap();
    run(ctx.dir.path(), &["git", "add", "."]);
    snapshot!(ctx, "jj<shift+K>y");
}

#[test]
fn branch_selected_confirm() {
    snapshot!(setup(), "<shift+Y>jj<shift+K>");
}

#[test]
fn branch_selected() {
    snapshot!(setup(), "<shift+Y>jj<shift+K>y");
}

#[test]
fn unmerged_branch_selected_unmerged_confirm() {
    snapshot!(setup(), "<shift+Y>jjj<shift+K>y");
}

#[test]
fn unmerged_branch_selected() {
    snapshot!(setup(), "<shift+Y>jjj<shift+K>yy");
}
