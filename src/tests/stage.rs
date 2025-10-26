use super::*;

#[test]
fn staged_file() {
    let mut ctx = setup_init!();
    run(&ctx.dir, &["touch", "new-file"]);
    run(&ctx.dir, &["git", "add", "new-file"]);

    ctx.init_app();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn stage_all_unstaged() {
    let ctx = setup_init!();
    commit(&ctx.dir, "firstfile", "testing\ntesttest\n");
    commit(&ctx.dir, "secondfile", "testing\ntesttest\n");

    fs::write(ctx.dir.join("firstfile"), "blahonga\n").unwrap();
    fs::write(ctx.dir.join("secondfile"), "blahonga\n").unwrap();
    snapshot!(ctx, "js");
}

#[test]
fn stage_all_untracked() {
    let ctx = setup_init!();
    run(&ctx.dir, &["touch", "file-a"]);
    run(&ctx.dir, &["touch", "file-b"]);
    snapshot!(ctx, "js");
}

#[test]
fn stage_removed_line() {
    let ctx = setup_init!();
    commit(&ctx.dir, "firstfile", "testing\ntesttest\n");
    fs::write(ctx.dir.join("firstfile"), "weehooo\nblrergh\n").unwrap();
    snapshot!(ctx, "jj<tab><ctrl+j><ctrl+j>s");
}

#[test]
fn stage_added_line() {
    let ctx = setup_init!();
    commit(&ctx.dir, "firstfile", "testing\ntesttest\n");
    fs::write(ctx.dir.join("firstfile"), "weehooo\nblrergh\n").unwrap();

    snapshot!(ctx, "jj<tab><ctrl+j><ctrl+j><ctrl+j><ctrl+j>s");
}

#[test]
fn stage_changes_crlf() {
    let ctx = setup_init!();
    commit(&ctx.dir, "testfile", "testing\r\ntesttest\r\n");
    fs::write(ctx.dir.join("testfile"), "test\r\ntesttest\r\n").expect("error writing to file");

    snapshot!(ctx, "jj<tab>");
}
