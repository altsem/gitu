use super::*;

fn setup(ctx: TestContext) -> TestContext {
    commit(&ctx.dir, "first", "third");
    commit(&ctx.dir, "first", "second");
    commit(&ctx.dir, "first", "first");
    ctx
}

#[test]
fn grep_changes_prompt() {
    snapshot!(setup(setup_clone!()), "l-G");
}

#[test]
fn grep_changes_set_example() {
    snapshot!(setup(setup_clone!()), "l-Gexample<enter>");
}

#[test]
fn grep_changes_second() {
    snapshot!(setup(setup_clone!()), "l-Gsecond<enter>l");
}

#[test]
fn grep_changes_no_match() {
    snapshot!(setup(setup_clone!()), "l-Gdoesntexist<enter>l");
}

#[test]
fn grep_changes_second_other() {
    snapshot!(setup(setup_clone!()), "l-Gsecond<enter>omain<enter>");
}
