use super::*;

#[test]
fn staged_file() {
    let mut ctx = TestContext::setup_init();
    run(ctx.dir.path(), &["touch", "new-file"]);
    run(ctx.dir.path(), &["git", "add", "new-file"]);

    ctx.init_state();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn stage_all_unstaged() {
    let ctx = TestContext::setup_init();
    commit(ctx.dir.path(), "firstfile", "testing\ntesttest\n");
    commit(ctx.dir.path(), "secondfile", "testing\ntesttest\n");

    fs::write(ctx.dir.child("firstfile"), "blahonga\n").unwrap();
    fs::write(ctx.dir.child("secondfile"), "blahonga\n").unwrap();
    snapshot!(ctx, "js");
}

#[test]
fn stage_all_untracked() {
    let ctx = TestContext::setup_init();
    run(ctx.dir.path(), &["touch", "file-a"]);
    run(ctx.dir.path(), &["touch", "file-b"]);
    snapshot!(ctx, "js");
}

#[test]
fn stage_removed_line() {
    let ctx = TestContext::setup_init();
    commit(ctx.dir.path(), "firstfile", "testing\ntesttest\n");
    fs::write(ctx.dir.child("firstfile"), "weehooo\nblrergh\n").unwrap();
    snapshot!(ctx, "jj<tab><ctrl+j><ctrl+j>s");
}

#[test]
fn stage_added_line() {
    let ctx = TestContext::setup_init();
    commit(ctx.dir.path(), "firstfile", "testing\ntesttest\n");
    fs::write(ctx.dir.child("firstfile"), "weehooo\nblrergh\n").unwrap();

    snapshot!(ctx, "jj<tab><ctrl+j><ctrl+j><ctrl+j><ctrl+j>s");
}

#[test]
fn stage_changes_crlf() {
    let ctx = TestContext::setup_init();
    commit(ctx.dir.path(), "testfile", "testing\r\ntesttest");
    fs::write(ctx.dir.child("testfile"), "test\r\ntesttest").expect("error writing to file");

    snapshot!(ctx, "jj<tab>");
}
