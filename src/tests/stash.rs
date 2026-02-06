use super::*;

fn snapshot_with_files(snapshot_name: &str, mut ctx: TestContext, keys_input: &str) {
    let before = [
        (
            "file1.txt",
            fs::read_to_string(ctx.dir.join("file1.txt")).unwrap(),
        ),
        (
            "file2.txt",
            fs::read_to_string(ctx.dir.join("file2.txt")).unwrap(),
        ),
    ];

    let mut app = ctx.init_app();
    ctx.update(&mut app, keys(keys_input));

    let after = [
        (
            "file1.txt",
            fs::read_to_string(ctx.dir.join("file1.txt")).unwrap(),
        ),
        (
            "file2.txt",
            fs::read_to_string(ctx.dir.join("file2.txt")).unwrap(),
        ),
    ];

    let mut out = ctx.redact_buffer();
    out.push_str("\n\n[files before]\n");
    for (name, content) in before {
        out.push_str(&format!("--- {name} ---\n{content}"));
    }
    out.push_str("\n[files after]\n");
    for (name, content) in after {
        out.push_str(&format!("--- {name} ---\n{content}"));
    }

    insta::assert_snapshot!(snapshot_name, out);
}

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

fn setup_stash_for_patch_apply(ctx: TestContext) -> TestContext {
    commit(
        &ctx.dir,
        "file1.txt",
        "one\ntwo\nthree\nfour\nfive\nsix\nseven\neight\nnine\nten\n",
    );
    commit(&ctx.dir, "file2.txt", "alpha\nbeta\ngamma\n");

    fs::write(ctx.dir.join("file1.txt"), "one\ntwo\ntwo-and-a-half\ntwo-and-three-quarters\nthree\nfour\nfive\nsix\nseven\neight\nnine\nTEN\n").unwrap();
    fs::write(ctx.dir.join("file2.txt"), "alpha\nbeta\ngamma\ndelta\n").unwrap();
    run(&ctx.dir, &["git", "stash", "save", "apply-stash"]);
    ctx
}

#[test]
pub(crate) fn stash_apply_hunk_as_patch() {
    let ctx = setup_stash_for_patch_apply(setup_clone!());
    let snapshot_name = function_name!().rsplit("::").next().unwrap();
    snapshot_with_files(snapshot_name, ctx, "jj<enter>a");
}

#[test]
pub(crate) fn stash_apply_file_as_patch() {
    let ctx = setup_stash_for_patch_apply(setup_clone!());
    let snapshot_name = function_name!().rsplit("::").next().unwrap();
    snapshot_with_files(snapshot_name, ctx, "jj<enter><alt+h>a");
}

#[test]
pub(crate) fn stash_apply_line_as_patch() {
    let ctx = setup_stash_for_patch_apply(setup_clone!());
    let snapshot_name = function_name!().rsplit("::").next().unwrap();
    snapshot_with_files(snapshot_name, ctx, "jj<enter><ctrl+j>a");
}

#[test]
pub(crate) fn stash_apply_selected() {
    let ctx = setup_stash_for_patch_apply(setup_clone!());
    let snapshot_name = function_name!().rsplit("::").next().unwrap();
    snapshot_with_files(snapshot_name, ctx, "jja");
}
