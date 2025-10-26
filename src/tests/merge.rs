use temp_env::with_var;

use super::*;

fn setup(ctx: TestContext) -> TestContext {
    run(&ctx.dir, &["git", "checkout", "-b", "other-branch"]);
    commit(&ctx.dir, "new-file", "hello");
    run(&ctx.dir, &["git", "checkout", "main"]);
    ctx
}

#[test]
fn merge_menu() {
    snapshot!(setup(setup_clone!()), "m");
}

#[test]
fn merge_prompt() {
    snapshot!(setup(setup_clone!()), "mm");
}

#[test]
fn merge_ff_only() {
    snapshot!(setup(setup_clone!()), "m-fmother-branch<enter>");
}

#[test]
fn merge_no_ff() {
    with_var("GIT_MERGE_AUTOEDIT", Some("no"), || {
        snapshot!(setup(setup_clone!()), "m-nmother-branch<enter>");
    });
}
