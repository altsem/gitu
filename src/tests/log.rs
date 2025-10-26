use super::*;

fn setup(ctx: TestContext) -> TestContext {
    commit(&ctx.dir, "third commit", "");
    commit(&ctx.dir, "second commit", "");
    commit(&ctx.dir, "first commit", "");
    ctx
}

#[test]
fn limit_prompt() {
    snapshot!(setup(setup_clone!()), "l-n-n");
}

#[test]
fn limit_set_10() {
    snapshot!(setup(setup_clone!()), "l-n-n10<enter>");
}

#[test]
fn limit_invalid() {
    snapshot!(setup(setup_clone!()), "l-n-nfff<enter>");
}

#[test]
fn limit_2_commits() {
    snapshot!(setup(setup_clone!()), "l-n-n2<enter>l");
}

#[test]
fn limit_2_commits_other() {
    snapshot!(setup(setup_clone!()), "l-n-n2<enter>l");
}

#[test]
fn grep_prompt() {
    snapshot!(setup(setup_clone!()), "l-F");
}

#[test]
fn grep_set_example() {
    snapshot!(setup(setup_clone!()), "l-Fexample<enter>");
}

#[test]
fn grep_second() {
    snapshot!(setup(setup_clone!()), "l-Fsecond<enter>l");
}

#[test]
fn grep_no_match() {
    snapshot!(setup(setup_clone!()), "l-Fdoesntexist<enter>l");
}

#[test]
fn grep_second_other() {
    snapshot!(setup(setup_clone!()), "l-Fsecond<enter>omain<enter>");
}

#[test]
fn log_other_prompt() {
    snapshot!(setup(setup_clone!()), "lljlo");
}

#[test]
fn log_other() {
    snapshot!(setup(setup_clone!()), "lljlo<enter>");
}

#[test]
fn log_other_input() {
    snapshot!(setup(setup_clone!()), "lomain~1<enter>");
}

#[test]
fn log_other_invalid() {
    snapshot!(setup(setup_clone!()), "lo <enter>");
}
