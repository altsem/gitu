use super::*;

fn setup_log() -> (TestContext, crate::state::State) {
    let mut ctx = TestContext::setup_clone(80, 20);
    commit(ctx.dir.path(), "this-should-be-at-the-top", "");
    commit(ctx.dir.path(), "this-should-not-be-visible", "");

    let state = ctx.init_state();
    (ctx, state)
}

#[test]
fn log_other_prompt() {
    let (mut ctx, mut state) = setup_log();

    state
        .update(
            &mut ctx.term,
            &[key('l'), key('l'), key('j'), key('l'), key('o')],
        )
        .unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn log_other() {
    let (mut ctx, mut state) = setup_log();

    state
        .update(
            &mut ctx.term,
            &[
                key('l'),
                key('l'),
                key('j'),
                key('l'),
                key('o'),
                key_code(KeyCode::Enter),
            ],
        )
        .unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn log_other_input() {
    let (mut ctx, mut state) = setup_log();

    state
        .update(
            &mut ctx.term,
            &[
                key('l'),
                key('o'),
                key('m'),
                key('a'),
                key('i'),
                key('n'),
                key('~'),
                key('1'),
                key_code(KeyCode::Enter),
            ],
        )
        .unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn log_other_invalid() {
    let (mut ctx, mut state) = setup_log();

    state
        .update(
            &mut ctx.term,
            &[key('l'), key('o'), key(' '), key_code(KeyCode::Enter)],
        )
        .unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
}
