use super::*;

#[test]
pub(crate) fn quit() {
    let app = snapshot!(TestContext::setup_init(), "q");
    assert!(app.state.quit);
}

#[test]
pub(crate) fn quit_from_menu() {
    let app = snapshot!(TestContext::setup_init(), "hq");
    assert!(!app.state.quit);
}

#[test]
pub(crate) fn confirm_quit_prompt() {
    let mut ctx = TestContext::setup_init();
    ctx.config().general.confirm_quit.enabled = true;

    let app = snapshot!(ctx, "q");
    assert!(!app.state.quit);
}

#[test]
pub(crate) fn confirm_quit() {
    let mut ctx = TestContext::setup_init();
    ctx.config().general.confirm_quit.enabled = true;

    let app = snapshot!(ctx, "qy");
    assert!(app.state.quit);
}
