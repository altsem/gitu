use super::*;

#[test]
fn pull_menu_no_remote_or_upstream_set() {
    let ctx = TestContext::setup_clone();
    run(ctx.dir.path(), &["git", "branch", "--unset-upstream"]);
    snapshot!(ctx, "F");
}

#[test]
fn pull_menu_existing_push_remote_and_upstream() {
    let ctx = TestContext::setup_clone();
    run(
        ctx.dir.path(),
        &["git", "config", "branch.main.pushRemote", "origin"],
    );
    snapshot!(ctx, "F");
}

#[test]
fn pull_upstream() {
    let ctx = TestContext::setup_clone();
    clone_and_commit(&ctx.remote_dir, "remote-file", "hello");
    snapshot!(ctx, "Fu");
}

#[test]
fn pull_push_remote() {
    let ctx = TestContext::setup_clone();
    run(
        ctx.dir.path(),
        &["git", "config", "branch.main.pushRemote", "origin"],
    );

    snapshot!(ctx, "Fp");
}

#[test]
fn pull_upstream_prompt() {
    let ctx = TestContext::setup_clone();
    run(ctx.dir.path(), &["git", "branch", "--unset-upstream"]);
    snapshot!(ctx, "Fu");
}

#[test]
fn pull_push_remote_prompt() {
    let ctx = TestContext::setup_clone();
    snapshot!(ctx, "Fp");
}

#[test]
fn pull_setup_upstream() {
    let ctx = TestContext::setup_clone();
    run(ctx.dir.path(), &["git", "checkout", "-b", "new-branch"]);
    snapshot!(ctx, "Fumain<enter>F");
}

#[test]
fn pull_setup_upstream_same_as_head() {
    let ctx = TestContext::setup_clone();
    run(ctx.dir.path(), &["git", "checkout", "-b", "new-branch"]);
    snapshot!(ctx, "Funew-branch<enter>");
}

#[test]
fn pull_setup_push_remote() {
    let ctx = TestContext::setup_clone();
    snapshot!(ctx, "Fporigin<enter>F");
}

#[test]
fn pull_from_elsewhere_prompt() {
    snapshot!(TestContext::setup_clone(), "Fe");
}

#[test]
fn pull_from_elsewhere() {
    snapshot!(TestContext::setup_clone(), "Feorigin<enter>");
}
