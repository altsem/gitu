use super::*;

fn setup_picker(ctx: TestContext) -> TestContext {
    // Create multiple branches and tags for comprehensive testing
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
    run(&ctx.dir, &["git", "tag", "v1.0.0"]);
    run(&ctx.dir, &["git", "tag", "v2.0.0"]);

    ctx
}

// ==================== Checkout Tests ====================

#[test]
fn checkout_picker() {
    snapshot!(setup_picker(setup_clone!()), "bb");
}

#[test]
fn checkout_picker_cancel() {
    snapshot!(setup_picker(setup_clone!()), "bb<esc>");
}

#[test]
fn checkout_select_from_list() {
    snapshot!(setup_picker(setup_clone!()), "bbfeature-a<enter>");
}

#[test]
fn checkout_use_custom_input() {
    let ctx = setup_picker(setup_clone!());
    // Get the commit hash of the first commit
    let output = run(&ctx.dir, &["git", "rev-parse", "HEAD"]);
    let commit_hash = output.trim();

    snapshot!(ctx, &format!("bb{}<enter>", commit_hash));
}

// ==================== Delete Tests ====================

#[test]
fn delete_picker() {
    snapshot!(setup_picker(setup_clone!()), "bK");
}

#[test]
fn delete_picker_cancel() {
    snapshot!(setup_picker(setup_clone!()), "bK<esc>");
}

#[test]
fn delete_select_from_list() {
    snapshot!(setup_picker(setup_clone!()), "bKfeature-a<enter>");
}

#[test]
fn delete_use_custom_input() {
    snapshot!(setup_picker(setup_clone!()), "bKfeature-b<enter>");
}

#[test]
fn delete_unmerged_branch() {
    let ctx = setup_picker(setup_clone!());
    snapshot!(ctx, "bKbugfix-123<enter>nbKbugfix-123<enter>y");
}

// ==================== CheckoutNewBranch Tests ====================

#[test]
fn checkout_new_branch() {
    snapshot!(setup_clone!(), "bcnew<enter>");
}

// ==================== Spinoff Tests ====================

#[test]
fn spinoff_branch() {
    snapshot!(setup_picker(setup_clone!()), "bsnew<enter>");
}

#[test]
fn spinoff_branch_with_unmerged_commits() {
    let ctx = setup_clone!();
    commit(&ctx.dir, "first commit", "");

    snapshot!(ctx, "bsnew<enter>");
}

#[test]
fn spinoff_existing_branch() {
    snapshot!(setup_picker(setup_clone!()), "bsfeature-a<enter>");
}

// ==================== Branch Menu Test ====================

#[test]
fn branch_menu() {
    snapshot!(setup_picker(setup_clone!()), "b");
}
