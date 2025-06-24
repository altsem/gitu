use super::*;

fn setup() -> TestContext {
    let ctx = TestContext::setup_clone();
    run(ctx.dir.path(), &["git", "checkout", "-b", "merged"]);
    run(ctx.dir.path(), &["git", "checkout", "-b", "unmerged"]);
    commit(ctx.dir.path(), "first commit", "");
    run(ctx.dir.path(), &["git", "checkout", "main"]);
    ctx
}

// <shift+Y>
#[test]
fn branch_menu() {
    snapshot!(setup(), "<shift+Y>jb");
}

#[test]
fn switch_branch_selected() {
    snapshot!(setup(), "<shift+Y>jjbb<enter>");
}

#[test]
fn switch_branch_input() {
    snapshot!(setup(), "<shift+Y>bbmerged<enter>");
}

#[test]
fn checkout_new_branch() {
    snapshot!(setup(), "bcnew<enter>");
}

#[test]
fn delete_branch_selected() {
    snapshot!(setup(), "<shift+Y>jjb<shift+K><enter>");
}

#[test]
fn delete_branch_input() {
    snapshot!(setup(), "b<shift+K>merged<enter>");
}

#[test]
fn delete_branch_empty() {
    snapshot!(setup(), "b<shift+K><enter>");
}

#[test]
fn delete_unmerged_branch() {
    snapshot!(
        setup(),
        "b<shift+K>unmerged<enter>nb<shift+K>unmerged<enter>y"
    );
}
