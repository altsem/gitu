use super::*;

fn setup_scroll() -> (TestContext, crate::app::App) {
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

    let mut app = ctx.init_app();
    ctx.update(&mut app, keys("jjjj<tab>k<tab>k<tab>"));
    (ctx, app)
}

#[test]
fn scroll_down() {
    let (mut ctx, mut app) = setup_scroll();
    ctx.update(&mut app, keys("<ctrl+d>"));
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn scroll_past_selection() {
    let (mut ctx, mut app) = setup_scroll();
    ctx.update(&mut app, keys("<ctrl+d><ctrl+d><ctrl+d>"));
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn move_prev_sibling() {
    let (mut ctx, mut app) = setup_scroll();
    ctx.update(&mut app, keys("<alt+k><alt+k>"));
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn move_next_sibling() {
    let (mut ctx, mut app) = setup_scroll();
    ctx.update(&mut app, keys("<alt+j>"));
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn move_next_then_parent_section() {
    let (mut ctx, mut app) = setup_scroll();
    ctx.update(&mut app, keys("<alt+j><alt+h>"));
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn exit_from_prompt_exits_menu() {
    snapshot!(TestContext::setup_init(), "bb<esc>");
}

#[test]
fn re_enter_prompt_from_menu() {
    snapshot!(TestContext::setup_init(), "bb<esc>bb");
}
