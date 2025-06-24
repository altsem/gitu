use super::*;

fn setup() -> TestContext {
    let ctx = TestContext::setup_clone();
    commit(ctx.dir.path(), "unwanted-file", "");
    ctx
}

#[test]
pub(crate) fn reset_menu() {
    snapshot!(setup(), "llj<shift+X>");
}

#[test]
pub(crate) fn reset_soft_prompt() {
    snapshot!(setup(), "llj<shift+X>sq");
}

#[test]
pub(crate) fn reset_soft() {
    snapshot!(setup(), "llj<shift+X>s<enter>q");
}

#[test]
pub(crate) fn reset_mixed() {
    snapshot!(setup(), "llj<shift+X>m<enter>q");
}

#[test]
fn reset_hard() {
    snapshot!(setup(), "llj<shift+X>h<enter>q");
}
