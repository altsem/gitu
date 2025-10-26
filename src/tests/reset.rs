use super::*;

fn setup(ctx: TestContext) -> TestContext {
    commit(&ctx.dir, "unwanted-file", "");
    ctx
}

#[test]
pub(crate) fn reset_menu() {
    snapshot!(setup(setup_clone!()), "lljX");
}

#[test]
pub(crate) fn reset_soft_prompt() {
    snapshot!(setup(setup_clone!()), "lljXsq");
}

#[test]
pub(crate) fn reset_soft() {
    snapshot!(setup(setup_clone!()), "lljXs<enter>q");
}

#[test]
pub(crate) fn reset_mixed() {
    snapshot!(setup(setup_clone!()), "lljXm<enter>q");
}

#[test]
fn reset_hard() {
    snapshot!(setup(setup_clone!()), "lljXh<enter>q");
}
