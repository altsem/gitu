use super::*;

#[test]
fn pull_upstream() {
    let ctx = TestContext::setup_clone();
    clone_and_commit(&ctx.remote_dir, "remote-file", "hello");
    snapshot!(ctx, "Fu");
}

#[test]
fn pull_upstream_prompt() {
    let ctx = TestContext::setup_init();
    snapshot!(ctx, "Fu");
}

#[test]
fn pull_from_elsewhere_prompt() {
    snapshot!(TestContext::setup_clone(), "Fe");
}

#[test]
fn pull_from_elsewhere() {
    snapshot!(TestContext::setup_clone(), "Feorigin<enter>");
}

#[test]
fn pull_push_remote_prompt() {
    let ctx = TestContext::setup_clone();
    snapshot!(ctx, "Fp");
}

#[test]
fn pull_push_remote() {
    let ctx = TestContext::setup_clone();
    snapshot!(ctx, "Fporigin<enter>");
}

#[test]
fn pull_set_push_remote_pull_again() {
    let ctx = TestContext::setup_clone();
    let mut ctx = ctx;
    let mut state = ctx.init_state();
    state
        .update(&mut ctx.term, &keys("Fporigin<enter>"))
        .unwrap();
    state.update(&mut ctx.term, &keys("Fp")).unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
}
