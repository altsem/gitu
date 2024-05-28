use super::*;

#[test]
fn apply_hunk_no_newline() {
    let ctx = TestContext::setup_init();
    commit(ctx.dir.path(), "file.txt", "bad text");
    run(ctx.dir.path(), &["git", "checkout", "-b", "other"]);
    commit(ctx.dir.path(), "file.txt", "good text");
    run(ctx.dir.path(), &["git", "checkout", "main"]);

    snapshot!(ctx, "loother<enter><enter>aqq");
}

#[test]
fn apply_hunk() {
    let ctx = TestContext::setup_init();
    commit(ctx.dir.path(), "file.txt", "bad text\n");
    run(ctx.dir.path(), &["git", "checkout", "-b", "other"]);
    commit(ctx.dir.path(), "file.txt", "good text\n");
    run(ctx.dir.path(), &["git", "checkout", "main"]);

    snapshot!(ctx, "loother<enter><enter>aqq");
}

#[test]
fn apply_line() {
    let ctx = TestContext::setup_init();
    commit(ctx.dir.path(), "file.txt", "bad text\nmore bad text\n");
    run(ctx.dir.path(), &["git", "checkout", "-b", "other"]);
    commit(ctx.dir.path(), "file.txt", "good text\nmore good text\n");
    run(ctx.dir.path(), &["git", "checkout", "main"]);

    snapshot!(ctx, "loother<enter><enter><ctrl+j>a");
}

#[test]
fn reverse_line() {
    let ctx = TestContext::setup_init();
    commit(ctx.dir.path(), "file.txt", "bad text\nmore bad text\n");

    snapshot!(ctx, "ll<enter><ctrl+j>v");
}
