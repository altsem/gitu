use super::*;

fn setup() -> TestContext {
    let ctx = TestContext::setup_clone();
    run(ctx.dir.path(), &["git", "checkout", "-b", "other-branch"]);
    run(ctx.dir.path(), &["git", "checkout", "main"]);
    commit(ctx.dir.path(), "new-file", "hello");
    run(ctx.dir.path(), &["git", "checkout", "other-branch"]);
    ctx
}

#[test]
fn rebase_menu() {
    snapshot!(setup(), "r");
}

#[test]
fn rebase_elsewhere_prompt() {
    snapshot!(setup(), "re");
}

#[test]
fn rebase_elsewhere() {
    snapshot!(setup(), "remain<enter>");
}
