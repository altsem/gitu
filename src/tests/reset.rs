use super::*;

fn setup() -> TestContext {
    let ctx = TestContext::setup_clone();
    commit(ctx.dir.path(), "unwanted-file", "");
    ctx
}

#[test]
pub(crate) fn reset_menu() {
    snapshot!(setup(), "lljX");
}

#[test]
pub(crate) fn reset_soft_prompt() {
    snapshot!(setup(), "lljXsq");
}

#[test]
pub(crate) fn reset_soft() {
    snapshot!(setup(), "lljXs<enter>q");
}

#[test]
pub(crate) fn reset_mixed() {
    snapshot!(setup(), "lljXm<enter>q");
}

#[test]
fn reset_hard() {
    snapshot!(setup(), "lljXh<enter>q");
}
