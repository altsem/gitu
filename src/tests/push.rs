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
fn push_elsewhere_prompt() {
    snapshot!(TestContext::setup_clone(), "Pe");
}

#[test]
fn push_elsewhere() {
    snapshot!(TestContext::setup_clone(), "Peorigin<enter>");
}
