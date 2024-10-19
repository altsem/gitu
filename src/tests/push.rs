use super::*;

#[test]
fn push_menu_no_remote_or_upstream_set() {
    let ctx = TestContext::setup_clone();
    run(ctx.dir.path(), &["git", "branch", "--unset-upstream"]);
    snapshot!(ctx, "P");
}

#[test]
fn push_menu_existing_push_remote_and_upstream() {
    let ctx = TestContext::setup_clone();
    run(
        ctx.dir.path(),
        &["git", "config", "branch.main.pushRemote", "origin"],
    );
    snapshot!(ctx, "P");
}

#[test]
fn push_upstream() {
    let ctx = TestContext::setup_clone();
    commit(ctx.dir.path(), "new-file", "");
    snapshot!(ctx, "Pu");
}

#[test]
fn push_push_remote() {
    let ctx = TestContext::setup_clone();
    run(
        ctx.dir.path(),
        &["git", "config", "branch.main.pushRemote", "origin"],
    );

    snapshot!(ctx, "Pp");
}

#[test]
fn push_upstream_prompt() {
    let ctx = TestContext::setup_clone();
    run(ctx.dir.path(), &["git", "branch", "--unset-upstream"]);
    snapshot!(ctx, "Pu");
}

#[test]
fn push_push_remote_prompt() {
    let ctx = TestContext::setup_clone();
    commit(ctx.dir.path(), "new-file", "");
    snapshot!(ctx, "Pp");
}

#[test]
fn push_setup_upstream() {
    let ctx = TestContext::setup_clone();
    run(ctx.dir.path(), &["git", "checkout", "-b", "new-branch"]);
    commit(ctx.dir.path(), "new-file", "");
    snapshot!(ctx, "Pumain<enter>P");
}

#[test]
fn push_setup_upstream_same_as_head() {
    let ctx = TestContext::setup_clone();
    run(ctx.dir.path(), &["git", "checkout", "-b", "new-branch"]);
    commit(ctx.dir.path(), "new-file", "");
    snapshot!(ctx, "Punew-branch<enter>");
}

#[test]
fn push_setup_push_remote() {
    let ctx = TestContext::setup_clone();
    snapshot!(ctx, "Pporigin<enter>P");
}

#[test]
fn force_push() {
    let ctx = TestContext::setup_clone();
    commit(ctx.dir.path(), "new-file", "");
    snapshot!(ctx, "P-fu");
}

#[test]
fn open_push_menu_after_dash_input() {
    let ctx = TestContext::setup_clone();
    commit(ctx.dir.path(), "new-file", "");
    snapshot!(ctx, "-P");
}

#[test]
fn push_menu_no_branch() {
    let ctx = TestContext::setup_clone();
    run(ctx.dir.path(), &["git", "checkout", ""]);
    // TODO `llbb<enter>q` is a hack to checkout a commit directly (detach HEAD)
    snapshot!(ctx, "llbb<enter>qP");
}

#[test]
fn push_elsewhere_prompt() {
    snapshot!(TestContext::setup_clone(), "Pe");
}

#[test]
fn push_elsewhere() {
    snapshot!(TestContext::setup_clone(), "Peorigin<enter>");
}
