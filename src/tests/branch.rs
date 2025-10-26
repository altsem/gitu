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
