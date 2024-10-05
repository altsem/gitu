use super::*;

#[test]
fn push() {
    let ctx = TestContext::setup_clone();
    commit(ctx.dir.path(), "new-file", "");
    snapshot!(ctx, "Pu");
}

#[test]
fn force_push() {
    let ctx = TestContext::setup_clone();
    commit(ctx.dir.path(), "new-file", "");
    snapshot!(ctx, "P-fu");
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
    snapshot!(ctx, "Pp");
}

#[test]
fn push_menu_no_branch() {
    let ctx = TestContext::setup_clone();
    run(ctx.dir.path(), &["git", "checkout", ""]);
    // TODO `llbb<enter>q` is a hack to checkout a commit directly (detach HEAD)
    snapshot!(ctx, "llbb<enter>qP");
}

#[test]
fn push_push_remote() {
    let ctx = TestContext::setup_clone();
    commit(ctx.dir.path(), "new-file", "");
    snapshot!(ctx, "Pporigin<enter>");
}

#[test]
fn push_set_push_remote_push_again() {
    let ctx = TestContext::setup_clone();
    commit(ctx.dir.path(), "new-file", "");
    let mut ctx = ctx;
    let mut state = ctx.init_state();
    state
        .update(&mut ctx.term, &keys("Pporigin<enter>"))
        .unwrap();
    commit(ctx.dir.path(), "new-file2", "");
    state.update(&mut ctx.term, &keys("Pp")).unwrap();
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
