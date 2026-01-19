use temp_env::with_var;

use super::*;

fn setup_branch(ctx: TestContext) -> TestContext {
    run(&ctx.dir, &["git", "checkout", "-b", "other-branch"]);
    commit(&ctx.dir, "new-file", "hello");
    run(&ctx.dir, &["git", "checkout", "main"]);
    ctx
}

fn setup_branches(ctx: TestContext) -> TestContext {
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

fn setup_branch_tag_same_name(ctx: TestContext) -> TestContext {
    // Create a branch named v1.0.0
    run(&ctx.dir, &["git", "checkout", "-b", "v1.0.0"]);
    commit(&ctx.dir, "branch commit", "");
    run(&ctx.dir, &["git", "checkout", "main"]);

    // Create a different branch with different content
    run(&ctx.dir, &["git", "checkout", "-b", "other"]);
    commit(&ctx.dir, "other commit", "");

    // Create a tag also named v1.0.0 pointing to this different commit
    run(&ctx.dir, &["git", "tag", "v1.0.0"]);
    run(&ctx.dir, &["git", "checkout", "main"]);

    ctx
}

fn setup_on_branch_with_same_tag(ctx: TestContext) -> TestContext {
    // Create and checkout a branch named v1.0.0
    run(&ctx.dir, &["git", "checkout", "-b", "v1.0.0"]);
    commit(&ctx.dir, "branch commit", "");

    // Create a tag also named v1.0.0 pointing to current commit
    run(&ctx.dir, &["git", "tag", "v1.0.0"]);

    // Stay on the v1.0.0 branch
    ctx
}

#[test]
fn merge_menu() {
    snapshot!(setup_branch(setup_clone!()), "m");
}

#[test]
fn merge_picker() {
    snapshot!(setup_branches(setup_clone!()), "mm");
}

#[test]
fn merge_picker_duplicate_branch_tag_names() {
    // Test that when branch and tag have the same name,
    // picker displays them with prefixes (heads/v1.0.0, tags/v1.0.0)
    snapshot!(setup_branch_tag_same_name(setup_clone!()), "mm");
}

#[test]
fn merge_picker_duplicate_names_select_branch() {
    // Test merging the branch when there's a duplicate name
    with_var("GIT_MERGE_AUTOEDIT", Some("no"), || {
        snapshot!(
            setup_branch_tag_same_name(setup_clone!()),
            "mmheads/v1.0.0<enter>"
        );
    });
}

#[test]
fn merge_picker_duplicate_names_select_tag() {
    // Test merging the tag when there's a duplicate name
    with_var("GIT_MERGE_AUTOEDIT", Some("no"), || {
        snapshot!(
            setup_branch_tag_same_name(setup_clone!()),
            "mmtags/v1.0.0<enter>"
        );
    });
}

#[test]
fn merge_picker_current_branch_with_same_tag_name() {
    // Test that when current branch and tag have the same name,
    // current branch is excluded from picker, only tag is shown
    snapshot!(setup_on_branch_with_same_tag(setup_clone!()), "mm");
}

#[test]
fn merge_picker_default_tag_with_same_branch_name() {
    // Test that when selecting a tag, and a branch with same name exists,
    // both are shown but default (tag) is not duplicated
    // Navigate to tags view and select v1.0.0 tag, then open merge picker
    snapshot!(
        setup_branch_tag_same_name(setup_clone!()),
        "Yjjjjjjjjmm"
    );
}

#[test]
fn merge_picker_default_branch_with_same_tag_name() {
    // Test that when selecting a branch, and a tag with same name exists,
    // both are shown but default (branch) is not duplicated
    // Navigate to branches view and select v1.0.0 branch, then open merge picker
    snapshot!(setup_branch_tag_same_name(setup_clone!()), "Yjjjmm");
}

#[test]
fn merge_picker_custom_input() {
    snapshot!(setup_branches(setup_clone!()), "mmHEAD~2");
}

#[test]
fn merge_picker_cancel() {
    snapshot!(setup_branches(setup_clone!()), "mm<esc>");
}

#[test]
fn merge_select_from_list() {
    // Select feature-a branch from the list and merge it
    with_var("GIT_MERGE_AUTOEDIT", Some("no"), || {
        snapshot!(setup_branches(setup_clone!()), "mmfeature-a<enter>");
    });
}

#[test]
fn merge_use_custom_input() {
    // Use custom input with full commit hash
    with_var("GIT_MERGE_AUTOEDIT", Some("no"), || {
        snapshot!(
            setup_branches(setup_clone!()),
            "mmb66a0bf82020d6a386e94d0fceedec1f817d20c7<enter>"
        );
    });
}

#[test]
fn merge_ff_only() {
    snapshot!(setup_branch(setup_clone!()), "m-fmother-branch<enter>");
}

#[test]
fn merge_no_ff() {
    with_var("GIT_MERGE_AUTOEDIT", Some("no"), || {
        snapshot!(setup_branch(setup_clone!()), "m-nmother-branch<enter>");
    });
}
