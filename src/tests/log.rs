use super::*;

fn setup_log() -> (TestContext, crate::state::State) {
    let mut ctx = TestContext::setup_clone();
    commit(ctx.dir.path(), "this-should-be-at-the-top", "");
    commit(ctx.dir.path(), "this-should-not-be-visible", "");

    let state = ctx.init_state();
    (ctx, state)
}

#[test]
fn log_other_prompt() {
    let (mut ctx, mut state) = setup_log();

    state.update(&mut ctx.term, &keys("lljlo")).unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn log_other() {
    let (mut ctx, mut state) = setup_log();

    state.update(&mut ctx.term, &keys("lljlo<enter>")).unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn log_other_input() {
    let (mut ctx, mut state) = setup_log();

    state
        .update(&mut ctx.term, &keys("lomain~1<enter>"))
        .unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn log_other_invalid() {
    let (mut ctx, mut state) = setup_log();

    state.update(&mut ctx.term, &keys("lo <enter>")).unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
}
