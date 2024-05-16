use super::*;

fn setup() -> TestContext {
    let ctx = TestContext::setup_clone();
    commit(ctx.dir.path(), "third commit", "");
    commit(ctx.dir.path(), "second commit", "");
    commit(ctx.dir.path(), "first commit", "");
    ctx
}

#[test]
fn limit_prompt() {
    snapshot!(setup(), "l-n-n");
}

#[test]
fn limit_set_10() {
    snapshot!(setup(), "l-n-n10<enter>");
}

#[test]
fn limit_invalid() {
    snapshot!(setup(), "l-n-nfff<enter>");
}

#[test]
fn limit_2_commits() {
    snapshot!(setup(), "l-n-n2<enter>l");
}

#[test]
fn limit_2_commits_other() {
    snapshot!(setup(), "l-n-n2<enter>l");
}

#[test]
fn grep_prompt() {
    snapshot!(setup(), "l-F");
}

#[test]
fn grep_set_example() {
    snapshot!(setup(), "l-Fexample<enter>");
}

#[test]
fn grep_second() {
    snapshot!(setup(), "l-Fsecond<enter>l");
}

#[test]
fn grep_second_other() {
    snapshot!(setup(), "l-Fsecond<enter>omain<enter>");
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
