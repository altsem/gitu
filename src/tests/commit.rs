use super::*;

#[test]
fn commit_instant_fixup() {
    let mut ctx = TestContext::setup_init();
    let mut state = ctx.init_state();

    commit(ctx.dir.path(), "instant_fixup.txt", "initial\n");
    commit(ctx.dir.path(), "instant_fixup.txt", "mistake\n");
    fs::write(ctx.dir.child("instant_fixup.txt"), "fixed\n").unwrap();
    run(ctx.dir.path(), &["git", "add", "."]);
    state.update(&mut ctx.term, &keys("gjjjjjcF")).unwrap();

    insta::assert_snapshot!(ctx.redact_buffer());
}
