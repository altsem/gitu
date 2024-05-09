use super::*;

fn setup() -> TestContext {
    let ctx = TestContext::setup_clone();
    commit(ctx.dir.path(), "this-should-be-at-the-top", "");
    commit(ctx.dir.path(), "this-should-not-be-visible", "");
    ctx
}

#[test]
fn log_n_prompt_show() {
    snapshot!(setup(), "l-n-n");
}

#[test]
fn log_n_prompt_valid() {
    snapshot!(setup(), "l-n-n10<enter>");
}

#[test]
fn log_n_prompt_invalid() {
    snapshot!(setup(), "l-n-nfff<enter>");
}

#[test]
fn log_grep_prompt_show() {
    snapshot!(setup(), "l-F");
}

#[test]
fn log_grep_prompt_valid() {
    snapshot!(setup(), "l-F");
}

#[test]
fn log_grep_prompt_invalid() {
    snapshot!(setup(), "l-Fui<enter>");
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
