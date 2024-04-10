use super::*;

fn setup() -> TestContext {
    let ctx = TestContext::setup_clone();
    fs::write(ctx.dir.child("file-one"), "blahonga\n").unwrap();
    fs::write(ctx.dir.child("file-two"), "blahonga\n").unwrap();
    run(ctx.dir.path(), &["git", "add", "file-one"]);
    ctx
}

#[test]
pub(crate) fn stash_menu() {
    snapshot!(setup(), "z");
}

#[test]
pub(crate) fn stash_prompt() {
    snapshot!(setup(), "zz");
}

#[test]
pub(crate) fn stash() {
    snapshot!(setup(), "zztest<enter>");
}

#[test]
pub(crate) fn stash_index_prompt() {
    snapshot!(setup(), "zi");
}

#[test]
pub(crate) fn stash_index() {
    snapshot!(setup(), "zitest<enter>");
}

#[test]
pub(crate) fn stash_working_tree_prompt() {
    snapshot!(setup(), "zw");
}

#[test]
pub(crate) fn stash_working_tree() {
    snapshot!(setup(), "zwtest<enter>");
}

#[test]
pub(crate) fn stash_working_tree_when_everything_is_staged() {
    snapshot!(setup(), "jszw");
}

#[test]
pub(crate) fn stash_working_tree_when_nothing_is_staged() {
    let ctx = TestContext::setup_clone();
    fs::write(ctx.dir.child("file-one"), "blahonga\n").unwrap();
    snapshot!(ctx, "zwtest<enter>");
}

#[test]
pub(crate) fn stash_keeping_index_prompt() {
    snapshot!(setup(), "zx");
}

#[test]
pub(crate) fn stash_keeping_index() {
    snapshot!(setup(), "zxtest<enter>");
}

fn setup_two_stashes() -> TestContext {
    let ctx = setup();
    run(
        ctx.dir.path(),
        &["git", "stash", "push", "--staged", "--message", "file-one"],
    );
    run(
        ctx.dir.path(),
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
    snapshot!(setup_two_stashes(), "zp");
}

#[test]
pub(crate) fn stash_pop() {
    snapshot!(setup_two_stashes(), "zp1<enter>");
}

#[test]
pub(crate) fn stash_pop_default() {
    snapshot!(setup_two_stashes(), "zp<enter>");
}

#[test]
pub(crate) fn stash_apply_prompt() {
    snapshot!(setup_two_stashes(), "za");
}

#[test]
pub(crate) fn stash_apply() {
    snapshot!(setup_two_stashes(), "za1<enter>");
}

#[test]
pub(crate) fn stash_apply_default() {
    snapshot!(setup_two_stashes(), "za<enter>");
}

#[test]
pub(crate) fn stash_drop_prompt() {
    snapshot!(setup_two_stashes(), "zk");
}

#[test]
pub(crate) fn stash_drop() {
    snapshot!(setup_two_stashes(), "zk1<enter>");
}

#[test]
pub(crate) fn stash_drop_default() {
    snapshot!(setup_two_stashes(), "zk<enter>");
}
