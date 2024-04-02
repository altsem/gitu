use super::*;

fn setup_rebase() -> (TestContext, crate::state::State) {
    let mut ctx = TestContext::setup_clone(80, 20);
    run(ctx.dir.path(), &["git", "checkout", "-b", "other-branch"]);
    run(ctx.dir.path(), &["git", "checkout", "main"]);
    commit(ctx.dir.path(), "new-file", "hello");
    run(ctx.dir.path(), &["git", "checkout", "other-branch"]);

    let state = ctx.init_state();
    (ctx, state)
}

#[test]
fn rebase_menu() {
    let (mut ctx, mut state) = setup_rebase();

    state.update(&mut ctx.term, &keys("r")).unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn rebase_elsewhere_prompt() {
    let (mut ctx, mut state) = setup_rebase();

    state.update(&mut ctx.term, &keys("re")).unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn rebase_elsewhere() {
    let (mut ctx, mut state) = setup_rebase();

    state.update(&mut ctx.term, &keys("remain<enter>")).unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
}
