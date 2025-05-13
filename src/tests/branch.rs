use super::*;

fn setup() -> TestContext {
    let ctx = TestContext::setup_clone();
    run(ctx.dir.path(), &["git", "checkout", "-b", "merged"]);
    run(ctx.dir.path(), &["git", "checkout", "-b", "unmerged"]);
    commit(ctx.dir.path(), "first commit", "");
    run(ctx.dir.path(), &["git", "checkout", "main"]);
    ctx
}

#[test]
fn branch_menu() {
    snapshot!(setup(), "Yjb");
}

#[test]
fn switch_branch_selected() {
    snapshot!(setup(), "Yjjbb<enter>");
}

#[test]
fn switch_branch_input() {
    snapshot!(setup(), "Ybbmerged<enter>");
}

#[test]
fn checkout_new_branch() {
    snapshot!(setup(), "bcnew<enter>");
}

#[test]
fn delete_branch_selected() {
    snapshot!(setup(), "YjjbK<enter>");
}

#[test]
fn delete_branch_input() {
    snapshot!(setup(), "bKmerged<enter>");
}

#[test]
fn delete_branch_empty() {
    snapshot!(setup(), "bK<enter>");
}

#[test]
fn delete_unmerged_branch() {
    // TODO: Remove <esc> once #368 is fixed
    snapshot!(setup(), "bKunmerged<enter>n<esc>bKunmerged<enter>y");
}
