use super::*;

#[test]
pub(crate) fn quit() {
    let state = snapshot!(TestContext::setup_init(), "q");
    assert!(state.quit);
}

#[test]
pub(crate) fn quit_from_menu() {
    let state = snapshot!(TestContext::setup_init(), "hq");
    assert!(!state.quit);
}

#[test]
pub(crate) fn confirm_quit_prompt() {
    let mut ctx = TestContext::setup_init();
    ctx.config().general.confirm_quit.enabled = true;

    let state = snapshot!(ctx, "q");
    assert!(!state.quit);
}

#[test]
pub(crate) fn confirm_quit() {
    let mut ctx = TestContext::setup_init();
    ctx.config().general.confirm_quit.enabled = true;

    let state = snapshot!(ctx, "qy");
    assert!(state.quit);
}
