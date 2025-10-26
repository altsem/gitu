use super::*;

fn setup(ctx: TestContext) -> TestContext {
    run(&ctx.dir, &["git", "checkout", "-b", "merged"]);
    run(&ctx.dir, &["git", "checkout", "-b", "unmerged"]);
    commit(&ctx.dir, "first commit", "");
    run(&ctx.dir, &["git", "checkout", "main"]);
    ctx
}

#[test]
pub(crate) fn discard_branch_confirm_prompt() {
    let ctx = setup_clone!();
    run(&ctx.dir, &["git", "branch", "asd"]);
    snapshot!(ctx, "YjK");
}

#[test]
pub(crate) fn discard_branch_yes() {
    let ctx = setup_clone!();
    run(&ctx.dir, &["git", "branch", "asd"]);
    snapshot!(ctx, "YjKy");
}

#[test]
pub(crate) fn discard_branch_no() {
    let ctx = setup_clone!();
    run(&ctx.dir, &["git", "branch", "asd"]);
    snapshot!(ctx, "YjKn");
}

#[test]
pub(crate) fn discard_untracked_file() {
    let ctx = setup_clone!();
    run(&ctx.dir, &["touch", "some-file"]);
    snapshot!(ctx, "jjKy");
}

#[test]
pub(crate) fn discard_untracked_staged_file() {
    let ctx = setup_clone!();
    run(&ctx.dir, &["touch", "some-file"]);
    run(&ctx.dir, &["git", "add", "some-file"]);
    snapshot!(ctx, "jsjKy");
}

#[test]
pub(crate) fn discard_file_move() {
    let ctx = setup_clone!();
    commit(&ctx.dir, "new-file", "hello");
    run(&ctx.dir, &["git", "mv", "new-file", "moved-file"]);

    snapshot!(ctx, "jjKy");
}

#[test]
pub(crate) fn discard_unstaged_delta() {
    let ctx = setup_clone!();
    commit(&ctx.dir, "file-one", "FOO\nBAR\n");
    fs::write(ctx.dir.join("file-one"), "blahonga\n").unwrap();
    snapshot!(ctx, "jjKy");
}

#[test]
pub(crate) fn discard_unstaged_hunk() {
    let ctx = setup_clone!();
    commit(&ctx.dir, "file-one", "FOO\nBAR\n");
    fs::write(ctx.dir.join("file-one"), "blahonga\n").unwrap();
    snapshot!(ctx, "jj<tab>jKy");
}

#[test]
pub(crate) fn discard_unstaged_line() {
    let ctx = setup_clone!();
    commit(&ctx.dir, "file-one", "FOO\nBAR\n");
    fs::write(ctx.dir.join("file-one"), "blahonga\n").unwrap();
    snapshot!(ctx, "jj<tab>j<ctrl+j>Ky<ctrl+j><ctrl+j>Ky");
}

#[test]
pub(crate) fn discard_staged_file() {
    let ctx = setup_clone!();
    commit(&ctx.dir, "file-one", "FOO\nBAR\n");
    fs::write(ctx.dir.join("file-one"), "blahonga\n").unwrap();
    run(&ctx.dir, &["git", "add", "."]);
    snapshot!(ctx, "jjKy");
}

#[test]
fn branch_selected_confirm() {
    snapshot!(setup(setup_clone!()), "YjjK");
}

#[test]
fn branch_selected() {
    snapshot!(setup(setup_clone!()), "YjjKy");
}

#[test]
fn unmerged_branch_selected_unmerged_confirm() {
    snapshot!(setup(setup_clone!()), "YjjjKy");
}

#[test]
fn unmerged_branch_selected() {
    snapshot!(setup(setup_clone!()), "YjjjKyy");
}
