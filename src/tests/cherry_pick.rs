use super::*;

fn setup_cherry_pickable(ctx: TestContext) -> TestContext {
    run(&ctx.dir, &["git", "checkout", "-b", "other-branch"]);
    commit(&ctx.dir, "cherry-file", "cherry content");
    run(&ctx.dir, &["git", "checkout", "main"]);
    ctx
}

fn setup_conflict(ctx: TestContext) -> TestContext {
    commit(&ctx.dir, "conflict-file", "hello");
    run(&ctx.dir, &["git", "checkout", "-b", "other-branch"]);
    commit(&ctx.dir, "conflict-file", "hey");
    run(&ctx.dir, &["git", "checkout", "main"]);
    commit(&ctx.dir, "conflict-file", "hi");
    run_ignore_status(&ctx.dir, &["git", "cherry-pick", "other-branch"]);
    ctx
}

#[test]
fn cherry_pick_menu() {
    snapshot!(setup_clone!(), "A");
}

#[test]
fn cherry_pick_prompt() {
    // Navigate to the log, select the initial commit, then open cherry-pick prompt
    snapshot!(setup_cherry_pickable(setup_clone!()), "llAA");
}

#[test]
fn cherry_pick_prompt_cancel() {
    snapshot!(setup_cherry_pickable(setup_clone!()), "llAA<esc>");
}

#[test]
fn cherry_pick() {
    snapshot!(
        setup_cherry_pickable(setup_clone!()),
        "llAAother-branch<enter>"
    );
}

#[test]
fn cherry_pick_no_commit() {
    snapshot!(
        setup_cherry_pickable(setup_clone!()),
        "llA-nAAother-branch<enter>"
    );
}

#[test]
fn cherry_pick_conflict_status() {
    let mut ctx = setup_conflict(setup_clone!());
    ctx.init_app();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn cherry_pick_abort() {
    snapshot!(setup_conflict(setup_clone!()), "Aa");
}

#[test]
fn cherry_pick_continue() {
    snapshot!(setup_conflict(setup_clone!()), "Ac");
}
