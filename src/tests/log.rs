use super::*;

fn setup() -> TestContext {
    let ctx = TestContext::setup_clone();
    commit(ctx.dir.path(), "this-should-be-at-the-top", "");
    commit(ctx.dir.path(), "this-should-not-be-visible", "");
    ctx
}

#[test]
fn log_other_prompt() {
    snapshot!(setup(), "lljlo");
}

#[test]
fn log_other() {
    snapshot!(setup(), "lljlo<enter>");
}

#[test]
fn log_other_input() {
    snapshot!(setup(), "lomain~1<enter>");
}

#[test]
fn log_other_invalid() {
    snapshot!(setup(), "lo <enter>");
}
