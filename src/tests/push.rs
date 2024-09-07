use super::*;

#[test]
fn push() {
    let ctx = TestContext::setup_clone();
    commit(ctx.dir.path(), "new-file", "");
    snapshot!(ctx, "Pp");
}

#[test]
fn force_push() {
    let ctx = TestContext::setup_clone();
    commit(ctx.dir.path(), "new-file", "");
    snapshot!(ctx, "P-fp");
}

#[test]
fn open_push_menu_after_dash_input() {
    let ctx = TestContext::setup_clone();
    commit(ctx.dir.path(), "new-file", "");
    snapshot!(ctx, "-P");
}

#[test]
fn push_push_remote_prompt() {
    let ctx = TestContext::setup_clone();
    commit(ctx.dir.path(), "new-file", "");
    snapshot!(ctx, "Pr");
}

#[test]
fn push_push_remote() {
    let ctx = TestContext::setup_clone();
    commit(ctx.dir.path(), "new-file", "");
    snapshot!(ctx, "Prorigin<enter>");
}

#[test]
fn push_set_push_remote_push_again() {
    let ctx = TestContext::setup_clone();
    commit(ctx.dir.path(), "new-file", "");
    let mut ctx = ctx;
    let mut state = ctx.init_state();
    state.update(&mut ctx.term, &keys("Prorigin<enter>")).unwrap();
    commit(ctx.dir.path(), "new-file2", "");
    state.update(&mut ctx.term, &keys("Pr")).unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn push_elsewhere_prompt() {
    snapshot!(TestContext::setup_clone(), "Pe");
}

#[test]
fn push_elsewhere() {
    snapshot!(TestContext::setup_clone(), "Peorigin<enter>");
}
