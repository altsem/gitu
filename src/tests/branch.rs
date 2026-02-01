use super::*;

fn setup(ctx: TestContext) -> TestContext {
    run(&ctx.dir, &["git", "checkout", "-b", "merged"]);
    run(&ctx.dir, &["git", "checkout", "-b", "unmerged"]);
    commit(&ctx.dir, "first commit", "");
    run(&ctx.dir, &["git", "checkout", "main"]);
    ctx
}

#[test]
fn branch_menu() {
    snapshot!(setup(setup_clone!()), "Yjb");
}

#[test]
fn switch_branch_selected() {
    snapshot!(setup(setup_clone!()), "Yjjbb<enter>");
}

#[test]
fn switch_branch_input() {
    snapshot!(setup(setup_clone!()), "Ybbmerged<enter>");
}

#[test]
fn switch_branch_picker() {
    snapshot!(setup(setup_clone!()), "bb");
}

#[test]
fn switch_branch_selected_revision_picker() {
    snapshot!(setup(setup_clone!()), "Yjjbb");
}

#[test]
fn checkout_new_branch() {
    snapshot!(setup(setup_clone!()), "bcnew<enter>");
}

#[test]
fn delete_branch_selected() {
    snapshot!(setup(setup_clone!()), "YjjbK<enter>");
}

#[test]
fn delete_branch_input() {
    snapshot!(setup(setup_clone!()), "bKmerged<enter>");
}

#[test]
fn delete_branch_empty() {
    snapshot!(setup(setup_clone!()), "bK<enter>");
}

#[test]
fn delete_unmerged_branch() {
    let ctx = setup(setup_clone!());
    snapshot!(ctx, "bKunmerged<enter>nbKunmerged<enter>y");
}

#[test]
fn spinoff_branch() {
    snapshot!(setup(setup_clone!()), "bsnew<enter>");
}

#[test]
fn spinoff_branch_with_unmerged_commits() {
    let ctx = setup_clone!();
    commit(&ctx.dir, "first commit", "");

    snapshot!(ctx, "bsnew<enter>");
}

#[test]
fn spinoff_existing_branch() {
    snapshot!(setup(setup_clone!()), "bsunmerged<enter>");
}
