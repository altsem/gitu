use super::*;

fn setup(ctx: TestContext) -> TestContext {
    fs::write(ctx.dir.join("file-one"), "blahonga\n").unwrap();
    fs::write(ctx.dir.join("file-two"), "blahonga\n").unwrap();
    run(&ctx.dir, &["git", "add", "file-one"]);
    ctx
}

#[test]
pub(crate) fn stash_menu() {
    snapshot!(setup(setup_clone!()), "z");
}

#[test]
pub(crate) fn stash_prompt() {
    snapshot!(setup(setup_clone!()), "zz");
}

#[test]
pub(crate) fn stash() {
    snapshot!(setup(setup_clone!()), "zztest<enter>");
}

#[test]
pub(crate) fn stash_index_prompt() {
    snapshot!(setup(setup_clone!()), "zi");
}

#[test]
pub(crate) fn stash_index() {
    snapshot!(setup(setup_clone!()), "zitest<enter>");
}

#[test]
pub(crate) fn stash_working_tree_prompt() {
    snapshot!(setup(setup_clone!()), "zw");
}

#[test]
pub(crate) fn stash_working_tree() {
    snapshot!(setup(setup_clone!()), "zwtest<enter>");
}

#[test]
pub(crate) fn stash_working_tree_when_everything_is_staged() {
    snapshot!(setup(setup_clone!()), "jszw");
}

#[test]
pub(crate) fn stash_working_tree_when_nothing_is_staged() {
    let ctx = setup_clone!();
    fs::write(ctx.dir.join("file-one"), "blahonga\n").unwrap();
    snapshot!(ctx, "zwtest<enter>");
}

#[test]
pub(crate) fn stash_keeping_index_prompt() {
    snapshot!(setup(setup_clone!()), "zx");
}

#[test]
pub(crate) fn stash_keeping_index() {
    snapshot!(setup(setup_clone!()), "zxtest<enter>");
}

fn setup_two_stashes(ctx: TestContext) -> TestContext {
    let ctx = setup(ctx);
    run(
        &ctx.dir,
        &["git", "stash", "push", "--staged", "--message", "file-one"],
    );
    run(
        &ctx.dir,
        &[
            "git",
            "stash",
            "push",
            "--include-untracked",
            "--message",
            "file-two",
        ],
    );
    ctx
}

#[test]
pub(crate) fn stash_pop_prompt() {
    snapshot!(setup_two_stashes(setup_clone!()), "zp");
}

#[test]
pub(crate) fn stash_pop() {
    snapshot!(setup_two_stashes(setup_clone!()), "zp1<enter>");
}

#[test]
pub(crate) fn stash_pop_default() {
    snapshot!(setup_two_stashes(setup_clone!()), "zp<enter>");
}

#[test]
pub(crate) fn stash_apply_prompt() {
    snapshot!(setup_two_stashes(setup_clone!()), "za");
}

#[test]
pub(crate) fn stash_apply() {
    snapshot!(setup_two_stashes(setup_clone!()), "za1<enter>");
}

#[test]
pub(crate) fn stash_apply_default() {
    snapshot!(setup_two_stashes(setup_clone!()), "za<enter>");
}

#[test]
pub(crate) fn stash_drop_prompt() {
    snapshot!(setup_two_stashes(setup_clone!()), "zk");
}

#[test]
pub(crate) fn stash_drop() {
    snapshot!(setup_two_stashes(setup_clone!()), "zk1<enter>");
}

#[test]
pub(crate) fn stash_drop_default() {
    snapshot!(setup_two_stashes(setup_clone!()), "zk<enter>");
}
