use super::*;

#[test]
fn pull_menu_no_remote_or_upstream_set() {
    let ctx = TestContext::setup_clone();
    run(ctx.dir.path(), &["git", "branch", "--unset-upstream"]);
    snapshot!(ctx, "<shift+F>");
}

#[test]
fn pull_menu_existing_push_remote_and_upstream() {
    let ctx = TestContext::setup_clone();
    run(
        ctx.dir.path(),
        &["git", "config", "branch.main.pushRemote", "origin"],
    );
    snapshot!(ctx, "<shift+F>");
}

#[test]
fn pull_upstream() {
    let ctx = TestContext::setup_clone();
    clone_and_commit(&ctx.remote_dir, "remote-file", "hello");
    snapshot!(ctx, "<shift+F>u");
}

#[test]
fn pull_push_remote() {
    let ctx = TestContext::setup_clone();
    run(
        ctx.dir.path(),
        &["git", "config", "branch.main.pushRemote", "origin"],
    );

    snapshot!(ctx, "<shift+F>p");
}

#[test]
fn pull_upstream_prompt() {
    let ctx = TestContext::setup_clone();
    run(ctx.dir.path(), &["git", "branch", "--unset-upstream"]);
    snapshot!(ctx, "<shift+F>u");
}

#[test]
fn pull_push_remote_prompt() {
    let ctx = TestContext::setup_clone();
    snapshot!(ctx, "<shift+F>p");
}

#[test]
fn pull_setup_upstream() {
    let ctx = TestContext::setup_clone();
    run(ctx.dir.path(), &["git", "checkout", "-b", "new-branch"]);
    snapshot!(ctx, "<shift+F>umain<enter><shift+F>");
}

#[test]
fn pull_setup_upstream_same_as_head() {
    let ctx = TestContext::setup_clone();
    run(ctx.dir.path(), &["git", "checkout", "-b", "new-branch"]);
    snapshot!(ctx, "<shift+F>unew-branch<enter>");
}

#[test]
fn pull_setup_push_remote() {
    let ctx = TestContext::setup_clone();
    snapshot!(ctx, "<shift+F>porigin<enter><shift+F>");
}

#[test]
fn pull_from_elsewhere_prompt() {
    snapshot!(TestContext::setup_clone(), "<shift+F>e");
}

#[test]
fn pull_from_elsewhere() {
    snapshot!(TestContext::setup_clone(), "<shift+F>eorigin<enter>");
}
