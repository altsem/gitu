use super::*;

fn setup() -> TestContext {
    let ctx = TestContext::setup_clone();
    run(ctx.dir.path(), &["git", "checkout", "-b", "other-branch"]);
    commit(ctx.dir.path(), "new-file", "hello");
    run(ctx.dir.path(), &["git", "checkout", "main"]);
    ctx
}

#[test]
fn merge_menu() {
    snapshot!(setup(), "m");
}

#[test]
fn merge_prompt() {
    snapshot!(setup(), "mm");
}

#[test]
fn merge_ff_only() {
    snapshot!(setup(), "m-fmother-branch<enter>");
}
