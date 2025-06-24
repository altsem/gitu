use super::*;

#[test]
fn push_menu_no_remote_or_upstream_set() {
    let ctx = TestContext::setup_clone();
    run(ctx.dir.path(), &["git", "branch", "--unset-upstream"]);
    snapshot!(ctx, "<shift+P>");
}

#[test]
fn push_menu_existing_push_remote_and_upstream() {
    let ctx = TestContext::setup_clone();
    run(
        ctx.dir.path(),
        &["git", "config", "branch.main.pushRemote", "origin"],
    );
    snapshot!(ctx, "<shift+P>");
}

#[test]
fn push_upstream() {
    let ctx = TestContext::setup_clone();
    commit(ctx.dir.path(), "new-file", "");
    snapshot!(ctx, "<shift+P>u");
}

#[test]
fn push_push_remote() {
    let ctx = TestContext::setup_clone();
    run(
        ctx.dir.path(),
        &["git", "config", "branch.main.pushRemote", "origin"],
    );

    snapshot!(ctx, "<shift+P>p");
}

#[test]
fn push_upstream_prompt() {
    let ctx = TestContext::setup_clone();
    run(ctx.dir.path(), &["git", "branch", "--unset-upstream"]);
    snapshot!(ctx, "<shift+P>u");
}

#[test]
fn push_push_remote_prompt() {
    let ctx = TestContext::setup_clone();
    commit(ctx.dir.path(), "new-file", "");
    snapshot!(ctx, "<shift+P>p");
}

#[test]
fn push_setup_upstream() {
    let ctx = TestContext::setup_clone();
    run(ctx.dir.path(), &["git", "checkout", "-b", "new-branch"]);
    commit(ctx.dir.path(), "new-file", "");
    snapshot!(ctx, "<shift+P>umain<enter><shift+P>");
}

#[test]
fn push_setup_upstream_same_as_head() {
    let ctx = TestContext::setup_clone();
    run(ctx.dir.path(), &["git", "checkout", "-b", "new-branch"]);
    commit(ctx.dir.path(), "new-file", "");
    snapshot!(ctx, "<shift+P>unew-branch<enter>");
}

#[test]
fn push_setup_push_remote() {
    let ctx = TestContext::setup_clone();
    snapshot!(ctx, "<shift+P>porigin<enter><shift+P>");
}

#[test]
fn force_push() {
    let ctx = TestContext::setup_clone();
    commit(ctx.dir.path(), "new-file", "");
    snapshot!(ctx, "<shift+P>-fu");
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
    snapshot!(ctx, "llbb<enter>q<shift+P>");
}

#[test]
fn push_elsewhere_prompt() {
    snapshot!(TestContext::setup_clone(), "<shift+P>e");
}

#[test]
fn push_elsewhere() {
    snapshot!(TestContext::setup_clone(), "<shift+P>eorigin<enter>");
}
