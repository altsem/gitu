use temp_env::with_var;

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

#[test]
fn merge_no_ff() {
    with_var("GIT_MERGE_AUTOEDIT", Some("no"), || {
        snapshot!(setup(), "m-nmother-branch<enter>");
    });
}
