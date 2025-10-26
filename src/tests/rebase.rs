use super::*;

fn setup(ctx: TestContext) -> TestContext {
    run(&ctx.dir, &["git", "checkout", "-b", "other-branch"]);
    run(&ctx.dir, &["git", "checkout", "main"]);
    commit(&ctx.dir, "new-file", "hello");
    run(&ctx.dir, &["git", "checkout", "other-branch"]);
    ctx
}

#[test]
fn rebase_menu() {
    snapshot!(setup(setup_clone!()), "r");
}

#[test]
fn rebase_elsewhere_prompt() {
    snapshot!(setup(setup_clone!()), "re");
}

#[test]
fn rebase_elsewhere() {
    snapshot!(setup(setup_clone!()), "remain<enter>");
}
