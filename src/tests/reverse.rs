use super::*;

fn snapshot_with_file(snapshot_name: &str, mut ctx: TestContext, filename: &str, keys_input: &str) {
    let before = fs::read_to_string(ctx.dir.join(filename)).unwrap();

    let mut app = ctx.init_app();
    ctx.update(&mut app, keys(keys_input));

    let after = fs::read_to_string(ctx.dir.join(filename)).unwrap();

    let mut out = ctx.redact_buffer();
    out.push_str("\n\n[file before]\n");
    out.push_str(&before);
    out.push_str("\n[file after]\n");
    out.push_str(&after);

    insta::assert_snapshot!(snapshot_name, out);
}

fn setup(ctx: TestContext) -> TestContext {
    commit(&ctx.dir, "file-one", "FOO\nBAR\nBAZ\n");
    fs::write(ctx.dir.join("file-one"), "blahonga\nBAR\nBAZ\n").unwrap();
    ctx
}

#[test]
pub(crate) fn reverse_unstaged_delta() {
    let ctx = setup(setup_clone!());
    let snapshot_name = function_name!().rsplit("::").next().unwrap();
    snapshot_with_file(snapshot_name, ctx, "file-one", "jjv");
}

#[test]
pub(crate) fn reverse_unstaged_hunk() {
    let ctx = setup(setup_clone!());
    let snapshot_name = function_name!().rsplit("::").next().unwrap();
    snapshot_with_file(snapshot_name, ctx, "file-one", "jj<tab>jv");
}

#[test]
pub(crate) fn reverse_unstaged_line() {
    let ctx = setup(setup_clone!());
    let snapshot_name = function_name!().rsplit("::").next().unwrap();
    snapshot_with_file(snapshot_name, ctx, "file-one", "jj<tab>j<ctrl+j>v");
}

#[test]
pub(crate) fn reverse_staged_delta() {
    let ctx = setup_staged(setup_clone!());
    let snapshot_name = function_name!().rsplit("::").next().unwrap();
    snapshot_with_file(snapshot_name, ctx, "file-one", "jjv");
}

#[test]
pub(crate) fn reverse_staged_hunk() {
    let ctx = setup_staged(setup_clone!());
    let snapshot_name = function_name!().rsplit("::").next().unwrap();
    snapshot_with_file(snapshot_name, ctx, "file-one", "jj<tab>jv");
}

#[test]
pub(crate) fn reverse_staged_line() {
    let ctx = setup_staged(setup_clone!());
    let snapshot_name = function_name!().rsplit("::").next().unwrap();
    snapshot_with_file(snapshot_name, ctx, "file-one", "jj<tab>j<ctrl+j>v");
}

fn setup_staged(ctx: TestContext) -> TestContext {
    commit(&ctx.dir, "file-one", "FOO\nBAR\nBAZ\n");
    fs::write(ctx.dir.join("file-one"), "blahonga\nBAR\nBAZ\n").unwrap();
    run(&ctx.dir, &["git", "add", "file-one"]);
    ctx
}
