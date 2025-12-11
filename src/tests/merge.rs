use temp_env::with_var;

use super::*;

fn setup(ctx: TestContext) -> TestContext {
    run(&ctx.dir, &["git", "checkout", "-b", "other-branch"]);
    commit(&ctx.dir, "new-file", "hello");
    run(&ctx.dir, &["git", "checkout", "main"]);
    ctx
}

fn setup_picker(ctx: TestContext) -> TestContext {
    // Create multiple branches for merging
    run(&ctx.dir, &["git", "checkout", "-b", "feature-a"]);
    commit(&ctx.dir, "feature-a commit", "");
    run(&ctx.dir, &["git", "checkout", "main"]);

    run(&ctx.dir, &["git", "checkout", "-b", "feature-b"]);
    commit(&ctx.dir, "feature-b commit", "");
    run(&ctx.dir, &["git", "checkout", "main"]);

    run(&ctx.dir, &["git", "checkout", "-b", "bugfix-123"]);
    commit(&ctx.dir, "bugfix commit", "");
    run(&ctx.dir, &["git", "checkout", "main"]);

    // Create some tags
    run(&ctx.dir, &["git", "tag", "v1.0.0", "feature-a"]);
    run(&ctx.dir, &["git", "tag", "v2.0.0", "feature-b"]);

    ctx
}

#[test]
fn merge_menu() {
    snapshot!(setup(setup_clone!()), "m");
}

#[test]
fn merge_picker() {
    snapshot!(setup_picker(setup_clone!()), "mm");
}

#[test]
fn merge_picker_custom_input() {
    snapshot!(setup_picker(setup_clone!()), "mmHEAD~2");
}

#[test]
fn merge_picker_cancel() {
    snapshot!(setup_picker(setup_clone!()), "mm<esc>");
}

#[test]
fn merge_select_from_list() {
    // Select feature-a branch from the list and merge it
    with_var("GIT_MERGE_AUTOEDIT", Some("no"), || {
        snapshot!(setup_picker(setup_clone!()), "mmfeature-a<enter>");
    });
}

#[test]
fn merge_use_custom_input() {
    // Use custom input with full commit hash
    with_var("GIT_MERGE_AUTOEDIT", Some("no"), || {
        snapshot!(
            setup_picker(setup_clone!()),
            "mmb66a0bf82020d6a386e94d0fceedec1f817d20c7<enter>"
        );
    });
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
