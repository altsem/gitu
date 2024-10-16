use super::*;

fn setup_scroll() -> (TestContext, crate::state::State) {
    let mut ctx = TestContext::setup_init();
    for file in ["file-1", "file-2", "file-3"] {
        commit(ctx.dir.path(), file, "");
        fs::write(
            ctx.dir.child(file),
            (1..=20).fold(String::new(), |mut acc, i| {
                use std::fmt::Write as _;

                writeln!(acc, "line {} ({})", i, file).unwrap();
                acc
            }),
        )
        .unwrap();
    }

    let mut state = ctx.init_state();
    state
        .update(&mut ctx.term, &keys("jjjj<tab>k<tab>k<tab>"))
        .unwrap();
    (ctx, state)
}

#[test]
fn scroll_down() {
    let (mut ctx, mut state) = setup_scroll();

    state.update(&mut ctx.term, &keys("<ctrl+d>")).unwrap();

    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn scroll_past_selection() {
    let (mut ctx, mut state) = setup_scroll();

    state
        .update(&mut ctx.term, &keys("<ctrl+d><ctrl+d><ctrl+d>"))
        .unwrap();

    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn move_prev_sibling() {
    let (mut ctx, mut state) = setup_scroll();
    state
        .update(&mut ctx.term, &keys("<alt+k><alt+k>"))
        .unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn move_next_sibling() {
    let (mut ctx, mut state) = setup_scroll();
    state.update(&mut ctx.term, &keys("<alt+j>")).unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn move_next_then_parent_section() {
    let (mut ctx, mut state) = setup_scroll();
    state
        .update(&mut ctx.term, &keys("<alt+j><alt+h>"))
        .unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn exit_from_prompt_shows_menu() {
    snapshot!(TestContext::setup_init(), "bb<esc>");
}

#[test]
fn re_enter_prompt_from_menu() {
    snapshot!(TestContext::setup_init(), "bb<esc>b");
}
