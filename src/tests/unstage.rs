use super::*;

#[test]
fn unstage_all_staged() {
    let ctx = TestContext::setup_init();
    run(ctx.dir.path(), &["touch", "one", "two", "unaffected"]);
    run(ctx.dir.path(), &["git", "add", "one", "two"]);
    snapshot!(ctx, "jjju");
}

#[test]
fn unstage_removed_line() {
    let ctx = TestContext::setup_init();
    commit(ctx.dir.path(), "firstfile", "testing\ntesttest\n");
    fs::write(ctx.dir.child("firstfile"), "weehooo\nblrergh\n").unwrap();
    run(ctx.dir.path(), &["git", "add", "."]);
    snapshot!(ctx, "jj<tab><ctrl+j><ctrl+j>u");
}

#[test]
fn unstage_added_line() {
    let ctx = TestContext::setup_init();
    commit(ctx.dir.path(), "firstfile", "testing\ntesttest\n");
    fs::write(ctx.dir.child("firstfile"), "weehooo\nblrergh\n").unwrap();
    run(ctx.dir.path(), &["git", "add", "."]);
    snapshot!(ctx, "jj<tab><ctrl+j><ctrl+j><ctrl+j><ctrl+j>u");
}
