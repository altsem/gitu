use super::*;

#[test]
fn commit_menu() {
    let ctx = setup_clone!();

    fs::write(ctx.dir.join("new_file.txt"), "lol\n").unwrap();
    run(&ctx.dir, &["git", "add", "."]);

    snapshot!(ctx, "c");
}

#[test]
fn commit_instant_fixup() {
    let mut ctx = setup_clone!();
    let mut state = ctx.init_app();

    commit(&ctx.dir, "instant_fixup.txt", "initial\n");
    commit(&ctx.dir, "instant_fixup.txt", "mistake\n");
    fs::write(ctx.dir.join("instant_fixup.txt"), "fixed\n").unwrap();
    run(&ctx.dir, &["git", "add", "."]);
    ctx.update(&mut state, keys("gjjjjjcF"));

    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn commit_instant_fixup_stashes_changes_and_keeps_empty() {
    let mut ctx = setup_clone!();
    let mut state = ctx.init_app();

    commit(&ctx.dir, "instant_fixup.txt", "initial\n");
    commit(&ctx.dir, "instant_fixup.txt", "mistake\n");
    run(
        &ctx.dir,
        &["git", "commit", "--allow-empty", "-m", "empty commit"],
    );
    fs::write(ctx.dir.join("instant_fixup.txt"), "fixed\n").unwrap();
    run(&ctx.dir, &["git", "add", "."]);
    fs::write(ctx.dir.join("instant_fixup.txt"), "unstaged\n").unwrap();
    ctx.update(&mut state, keys("gjjjjjjjjjcF"));

    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn commit_extend() {
    let ctx = setup_clone!();

    fs::write(ctx.dir.join("new_file.txt"), "lol\n").unwrap();
    run(&ctx.dir, &["git", "add", "."]);

    snapshot!(ctx, "ce");
}
