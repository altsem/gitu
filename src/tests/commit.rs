use super::*;

#[test]
fn commit_menu() {
    let ctx = TestContext::setup_clone();

    fs::write(ctx.dir.child("new_file.txt"), "lol\n").unwrap();
    run(ctx.dir.path(), &["git", "add", "."]);

    snapshot!(ctx, "c");
}

#[test]
fn commit_instant_fixup() {
    let mut ctx = TestContext::setup_init();
    let mut state = ctx.init_app();

    commit(ctx.dir.path(), "instant_fixup.txt", "initial\n");
    commit(ctx.dir.path(), "instant_fixup.txt", "mistake\n");
    fs::write(ctx.dir.child("instant_fixup.txt"), "fixed\n").unwrap();
    run(ctx.dir.path(), &["git", "add", "."]);
    ctx.update(&mut state, keys("gjjjjjcF"));

    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn commit_instant_fixup_stashes_changes_and_keeps_empty() {
    let mut ctx = TestContext::setup_init();
    let mut state = ctx.init_app();

    commit(ctx.dir.path(), "instant_fixup.txt", "initial\n");
    commit(ctx.dir.path(), "instant_fixup.txt", "mistake\n");
    run(
        ctx.dir.path(),
        &["git", "commit", "--allow-empty", "-m", "empty commit"],
    );
    fs::write(ctx.dir.child("instant_fixup.txt"), "fixed\n").unwrap();
    run(ctx.dir.path(), &["git", "add", "."]);
    fs::write(ctx.dir.child("instant_fixup.txt"), "unstaged\n").unwrap();
    ctx.update(&mut state, keys("gjjjjjjjjjcF"));

    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn commit_extend() {
    let ctx = TestContext::setup_clone();

    fs::write(ctx.dir.child("new_file.txt"), "lol\n").unwrap();
    run(ctx.dir.path(), &["git", "add", "."]);

    snapshot!(ctx, "ce");
}
