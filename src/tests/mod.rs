//! This module contains integration tests for Gitu.
//! Each test:
//! - sets up a temporary git repository in a temporary directory
//! - runs some commands
//! - asserts the output (`cargo insta` is used a lot https://insta.rs)
//! - cleans up the temporary directory
//!
//! It is useful when debugging to sometimes manually inspect a test-case.
//! ```rust`
//! dbg!(&ctx.dir.path());
//! ctx.dir.leak();
//! ````
//!

use std::fs;

#[macro_use]
mod helpers;
mod arg;
mod branch;
mod commit;
mod discard;
mod editor;
mod fetch;
mod log;
mod merge;
mod pull;
mod push;
mod quit;
mod rebase;
mod remote;
mod reset;
mod stage;
mod stash;
mod unstage;

use crossterm::event::MouseButton;
use helpers::{TestContext, clone_and_commit, commit, keys, mouse_event, mouse_scroll_event, run};

#[test]
fn no_repo() {
    let mut ctx = TestContext::setup_init();

    ctx.init_app();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn help_menu() {
    let mut ctx = TestContext::setup_init();

    let mut app = ctx.init_app();
    ctx.update(&mut app, keys("h"));
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn fresh_init() {
    let mut ctx = TestContext::setup_init();

    ctx.init_app();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn new_file() {
    let mut ctx = TestContext::setup_init();
    run(ctx.dir.path(), &["touch", "new-file"]);

    ctx.init_app();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn deleted_file() {
    let ctx = TestContext::setup_init();
    commit(ctx.dir.path(), "new-file", "testing\ntesttest\n");
    run(ctx.dir.path(), &["rm", "new-file"]);
    snapshot!(ctx, "");
}

#[test]
fn copied_file() {
    let ctx = TestContext::setup_init();
    commit(ctx.dir.path(), "new-file", "testing\ntesttest\n");
    run(ctx.dir.path(), &["cp", "new-file", "copied-file"]);
    run(ctx.dir.path(), &["git", "add", "-N", "."]);
    snapshot!(ctx, "");
}

#[test]
fn unstaged_changes() {
    let ctx = TestContext::setup_init();
    commit(ctx.dir.path(), "testfile", "testing\ntesttest\n");
    fs::write(ctx.dir.child("testfile"), "test\ntesttest\n").expect("error writing to file");
    snapshot!(ctx, "jj<tab>");
}

#[test]
fn binary_file() {
    let ctx = TestContext::setup_init();
    fs::write(ctx.dir.child("binary-file"), [0, 255]).expect("error writing to file");
    run(ctx.dir.path(), &["git", "add", "."]);
    snapshot!(ctx, "jj<tab>");
}

#[test]
fn non_ascii_filename() {
    let ctx = TestContext::setup_init();
    commit(ctx.dir.path(), "höhöhö", "hehehe\n");
    fs::write(ctx.dir.child("höhöhö"), "hahaha\n").expect("error writing to file");
    snapshot!(ctx, "jj<tab>");
}

#[test]
fn collapsed_sections_config() {
    let mut ctx = TestContext::setup_clone();
    ctx.config().general.collapsed_sections = vec![
        "untracked".into(),
        "recent_commits".into(),
        "branch_status".into(),
        // TODO rebase / revert/ merge conlict?
    ];
    fs::write(ctx.dir.child("untracked_file.txt"), "").unwrap();

    snapshot!(ctx, "");
}

#[test]
fn stash_list_with_limit() {
    let mut ctx = TestContext::setup_clone();
    ctx.config().general.stash_list_limit = 2;

    fs::write(ctx.dir.child("file1.txt"), "content").unwrap();
    run(ctx.dir.path(), &["git", "add", "."]);
    run(ctx.dir.path(), &["git", "stash", "save", "firststash"]);
    fs::write(ctx.dir.child("file2.txt"), "content").unwrap();
    run(ctx.dir.path(), &["git", "add", "."]);
    run(ctx.dir.path(), &["git", "stash", "save", "secondstash"]);
    fs::write(ctx.dir.child("file3.txt"), "content").unwrap();
    run(ctx.dir.path(), &["git", "add", "."]);
    run(ctx.dir.path(), &["git", "stash", "save", "thirdstash"]);

    snapshot!(ctx, "");
}

#[test]
fn recent_commits_with_limit() {
    let mut ctx = TestContext::setup_clone();
    ctx.config().general.recent_commits_limit = 2;
    commit(ctx.dir.path(), "firstfile", "testing\ntesttest\n");
    commit(ctx.dir.path(), "secondfile", "testing\ntesttest\n");
    commit(ctx.dir.path(), "thirdfile", "testing\ntesttest\n");
    snapshot!(ctx, "");
}

#[test]
fn log() {
    let ctx = TestContext::setup_clone();
    commit(ctx.dir.path(), "firstfile", "testing\ntesttest\n");
    run(ctx.dir.path(), &["git", "tag", "-am", ".", "annotated"]);
    commit(ctx.dir.path(), "secondfile", "testing\ntesttest\n");
    run(ctx.dir.path(), &["git", "tag", "a-tag"]);
    snapshot!(ctx, "ll");
}

#[test]
fn show() {
    let ctx = TestContext::setup_clone();
    commit(ctx.dir.path(), "firstfile", "This should be visible\n");
    snapshot!(ctx, "ll<enter>");
}

#[test]
fn show_stash() {
    let ctx = TestContext::setup_clone();

    fs::write(ctx.dir.child("file1.txt"), "content").unwrap();
    run(ctx.dir.path(), &["git", "add", "file1.txt"]);
    // Unstaged changes to "file1.txt"
    fs::write(ctx.dir.child("file1.txt"), "content\nmodified content").unwrap();
    run(ctx.dir.path(), &["git", "stash", "save", "firststash"]);

    snapshot!(ctx, "jj<enter>");
}

#[test]
fn rebase_conflict() {
    let mut ctx = TestContext::setup_clone();
    commit(ctx.dir.path(), "new-file", "hello");

    run(ctx.dir.path(), &["git", "checkout", "-b", "other-branch"]);
    commit(ctx.dir.path(), "new-file", "hey");

    run(ctx.dir.path(), &["git", "checkout", "main"]);
    commit(ctx.dir.path(), "new-file", "hi");

    run(ctx.dir.path(), &["git", "checkout", "other-branch"]);
    run(ctx.dir.path(), &["git", "rebase", "main"]);

    ctx.init_app();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn merge_conflict() {
    let mut ctx = TestContext::setup_clone();
    commit(ctx.dir.path(), "new-file", "hello");
    commit(ctx.dir.path(), "new-file-2", "hello");

    run(ctx.dir.path(), &["git", "checkout", "-b", "other-branch"]);
    commit(ctx.dir.path(), "new-file", "hey");
    commit(ctx.dir.path(), "new-file-2", "hey");

    run(ctx.dir.path(), &["git", "checkout", "main"]);
    commit(ctx.dir.path(), "new-file", "hi");
    commit(ctx.dir.path(), "new-file-2", "hi");

    run(ctx.dir.path(), &["git", "merge", "other-branch"]);

    ctx.init_app();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn revert_conflict() {
    let mut ctx = TestContext::setup_clone();
    commit(ctx.dir.path(), "new-file", "hey");
    commit(ctx.dir.path(), "new-file", "hi");

    run(ctx.dir.path(), &["git", "revert", "HEAD~1"]);

    ctx.init_app();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn revert_abort() {
    let ctx = TestContext::setup_clone();
    commit(ctx.dir.path(), "new-file", "hey");
    commit(ctx.dir.path(), "new-file", "hi");

    run(ctx.dir.path(), &["git", "revert", "HEAD~1"]);

    snapshot!(ctx, "Va");
}

#[test]
fn revert_menu() {
    let ctx = TestContext::setup_clone();
    snapshot!(ctx, "llV");
}

#[test]
fn revert_commit_prompt() {
    let ctx = TestContext::setup_clone();
    snapshot!(ctx, "llVV");
}

#[test]
fn revert_commit() {
    let ctx = TestContext::setup_clone();
    snapshot!(ctx, "llV-EV<enter>");
}

#[test]
fn moved_file() {
    let mut ctx = TestContext::setup_clone();
    commit(ctx.dir.path(), "new-file", "hello");
    run(ctx.dir.path(), &["git", "mv", "new-file", "moved-file"]);

    ctx.init_app();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn hide_untracked() {
    let mut ctx = TestContext::setup_clone();
    run(ctx.dir.path(), &["touch", "i-am-untracked"]);

    let mut app = ctx.init_app();
    let mut config = app.state.repo.config().unwrap();
    config.set_str("status.showUntrackedFiles", "off").unwrap();

    ctx.update(&mut app, keys("g"));
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn new_commit() {
    let mut ctx = TestContext::setup_clone();
    commit(ctx.dir.path(), "new-file", "");

    ctx.init_app();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn fetch_all() {
    let ctx = TestContext::setup_clone();
    clone_and_commit(&ctx.remote_dir, "remote-file", "hello");
    snapshot!(ctx, "fa");
}

mod show_refs {
    use super::*;

    #[test]
    fn show_refs_at_local_branch() {
        let ctx = TestContext::setup_clone();
        run(ctx.dir.path(), &["git", "tag", "main"]);
        snapshot!(ctx, "Y");
    }

    #[test]
    fn show_refs_at_remote_branch() {
        let ctx = TestContext::setup_clone();
        snapshot!(ctx, "Yjjjjbb<enter>Y");
    }

    #[test]
    fn show_refs_at_tag() {
        let ctx = TestContext::setup_clone();
        run(ctx.dir.path(), &["git", "tag", "v1.0"]);
        snapshot!(ctx, "Yjjjjjjbb<enter>Y");
    }
}

#[test]
fn updated_externally() {
    let mut ctx = TestContext::setup_init();
    fs::write(ctx.dir.child("b"), "test\n").unwrap();

    let mut app = ctx.init_app();
    ctx.update(&mut app, keys("jjsj"));

    fs::write(ctx.dir.child("a"), "test\n").unwrap();

    ctx.update(&mut app, keys("g"));
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn stage_last_hunk_of_first_delta() {
    let ctx = TestContext::setup_clone();
    commit(ctx.dir.path(), "file-one", "asdf\nblahonga\n");
    commit(ctx.dir.path(), "file-two", "FOO\nBAR\n");
    fs::write(ctx.dir.child("file-one"), "blahonga\n").unwrap();
    fs::write(ctx.dir.child("file-two"), "blahonga\n").unwrap();

    snapshot!(ctx, "jj<tab>js");
}

#[test]
fn go_down_past_collapsed() {
    let ctx = TestContext::setup_init();
    commit(ctx.dir.path(), "file-one", "asdf\nblahonga\n");
    commit(ctx.dir.path(), "file-two", "FOO\nBAR\n");
    fs::write(ctx.dir.child("file-one"), "blahonga\n").unwrap();
    fs::write(ctx.dir.child("file-two"), "blahonga\n").unwrap();

    snapshot!(ctx, "jjj");
}

#[test]
fn inside_submodule() {
    let mut ctx = TestContext::setup_clone();
    run(
        ctx.dir.path(),
        &[
            "git",
            "-c",
            "protocol.file.allow=always",
            "submodule",
            "add",
            ctx.remote_dir.path().to_str().unwrap(),
            "test-submodule",
        ],
    );

    let _app = ctx.init_app_at_path(ctx.dir.child("test-submodule"));
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn syntax_highlighted() {
    let ctx = TestContext::setup_init();
    commit(
        ctx.dir.path(),
        "syntax-highlighted.rs",
        "fn main() {\n    println!(\"Hey\");\n}\n",
    );
    fs::write(
        ctx.dir.child("syntax-highlighted.rs"),
        "fn main() {\n    println!(\"Bye\");\n}\n",
    )
    .unwrap();

    snapshot!(ctx, "jj<tab>");
}

#[test]
fn crlf_diff() {
    let mut ctx = TestContext::setup_init();
    let mut app = ctx.init_app();
    app.state
        .repo
        .config()
        .unwrap()
        .set_bool("core.autocrlf", true)
        .unwrap();

    commit(ctx.dir.path(), "crlf.txt", "unchanged\r\nunchanged\r\n");
    fs::write(ctx.dir.child("crlf.txt"), "unchanged\r\nchanged\r\n").unwrap();
    ctx.update(&mut app, keys("g"));

    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn tab_diff() {
    let mut ctx = TestContext::setup_init();
    let mut app = ctx.init_app();

    commit(ctx.dir.path(), "tab.txt", "this has no tab prefixed\n");
    fs::write(ctx.dir.child("tab.txt"), "\tthis has a tab prefixed\n").unwrap();
    ctx.update(&mut app, keys("g"));

    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn ext_diff() {
    let mut ctx = TestContext::setup_init();
    let mut app = ctx.init_app();

    fs::write(ctx.dir.child("unstaged.txt"), "unstaged\n").unwrap();
    fs::write(ctx.dir.child("staged.txt"), "staged\n").unwrap();
    run(ctx.dir.path(), &["git", "add", "-N", "unstaged.txt"]);
    run(ctx.dir.path(), &["git", "add", "staged.txt"]);
    run(
        ctx.dir.path(),
        &["git", "config", "diff.external", "/dev/null"],
    );
    ctx.update(&mut app, keys("g"));

    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn mouse_select_item() {
    let mut ctx = TestContext::setup_init();
    ctx.config().general.mouse_support = true;

    commit(ctx.dir.path(), "testfile", "testing\ntesttest\n");

    let mut app = ctx.init_app();
    ctx.update(&mut app, vec![mouse_event(0, 4, MouseButton::Left)]);
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn mouse_select_hunk_line() {
    let mut ctx = TestContext::setup_init();
    ctx.config().general.mouse_support = true;

    fs::write(ctx.dir.child("testfile"), "test\ntesttest\n").expect("error writing to file");
    run(ctx.dir.path(), &["git", "add", "."]);
    fs::write(ctx.dir.child("testfile"), "test\nmoretest\n").expect("error writing to file");

    let mut app = ctx.init_app();
    ctx.update(
        &mut app,
        vec![
            // Select the file with an unstaged change.
            mouse_event(0, 4, MouseButton::Left),
            // Expand the selected file.
            mouse_event(0, 4, MouseButton::Left),
            // Select the hunk line.
            mouse_event(0, 8, MouseButton::Left),
        ],
    );
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn mouse_select_ignore_empty_lines() {
    let mut ctx = TestContext::setup_init();
    ctx.config().general.mouse_support = true;

    commit(ctx.dir.path(), "testfile", "testing\ntesttest\n");
    fs::write(ctx.dir.child("testfile"), "test\nmoretest\n").expect("error writing to file");

    let mut app = ctx.init_app();
    ctx.update(
        &mut app,
        vec![
            // Click the last unstaged change.
            mouse_event(0, 4, MouseButton::Left),
            // Click the space underneath the last unstaged change.
            mouse_event(0, 5, MouseButton::Left),
        ],
    );
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn mouse_select_ignore_empty_region() {
    let mut ctx = TestContext::setup_init();
    ctx.config().general.mouse_support = true;

    commit(ctx.dir.path(), "testfile", "testing\ntesttest\n");
    fs::write(ctx.dir.child("testfile"), "test\nmoretest\n").expect("error writing to file");

    let mut app = ctx.init_app();
    ctx.update(
        &mut app,
        vec![
            // Click the last unstaged change.
            mouse_event(0, 4, MouseButton::Left),
            // Click the open space at the bottom of the screen.
            mouse_event(0, 10, MouseButton::Left),
        ],
    );
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn mouse_toggle_selected_item() {
    let mut ctx = TestContext::setup_init();
    ctx.config().general.mouse_support = true;

    commit(ctx.dir.path(), "testfile", "testing\ntesttest\n");
    fs::write(ctx.dir.child("testfile"), "test\nmoretest\n").expect("error writing to file");

    let mut app = ctx.init_app();
    ctx.update(
        &mut app,
        vec![
            // Click the last unstaged change.
            mouse_event(0, 4, MouseButton::Left),
            // Click the last unstaged change.
            mouse_event(0, 4, MouseButton::Left),
        ],
    );
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn mouse_show_item() {
    let mut ctx = TestContext::setup_init();
    ctx.config().general.mouse_support = true;

    commit(ctx.dir.path(), "testfile", "testing\ntesttest\n");

    let mut app = ctx.init_app();
    ctx.update(
        &mut app,
        vec![
            // Right-click the last unstaged change.
            mouse_event(0, 4, MouseButton::Right),
        ],
    );
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn mouse_show_ignore_empty_lines() {
    let mut ctx = TestContext::setup_init();
    ctx.config().general.mouse_support = true;

    commit(ctx.dir.path(), "testfile", "testing\ntesttest\n");
    fs::write(ctx.dir.child("testfile"), "test\nmoretest\n").expect("error writing to file");
    run(ctx.dir.path(), &["git", "add", "."]);
    run(ctx.dir.path(), &["git", "stash", "save", "firststash"]);

    let mut app = ctx.init_app();
    ctx.update(
        &mut app,
        vec![
            // Left-click the last stash.
            mouse_event(0, 4, MouseButton::Left),
            // Right-click the space underneath the last stash.
            mouse_event(0, 5, MouseButton::Right),
        ],
    );
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn mouse_show_ignore_empty_region() {
    let mut ctx = TestContext::setup_init();
    ctx.config().general.mouse_support = true;

    commit(ctx.dir.path(), "testfile", "testing\ntesttest\n");
    fs::write(ctx.dir.child("testfile"), "test\nmoretest\n").expect("error writing to file");
    run(ctx.dir.path(), &["git", "add", "."]);
    run(ctx.dir.path(), &["git", "stash", "save", "firststash"]);

    let mut app = ctx.init_app();
    ctx.update(
        &mut app,
        vec![
            // Left-click the last stash change.
            mouse_event(0, 4, MouseButton::Left),
            // Right-click the open space at the bottom of the screen.
            mouse_event(0, 10, MouseButton::Right),
        ],
    );
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn mouse_wheel_scroll_up() {
    let mut ctx = TestContext::setup_init();
    ctx.config().general.mouse_support = true;

    // Create many files to have something to scroll through
    for i in 1..=30 {
        let filename = format!("file{:02}", i);
        let content = "line 1\nline 2\nline 3\nline 4\nline 5\n".to_string();
        commit(ctx.dir.path(), &filename, &content);
        fs::write(
            ctx.dir.child(&filename),
            format!("modified content {}\n", i),
        )
        .expect("error writing to file");
    }

    let mut app = ctx.init_app();

    // Scroll down a bit to be able to scroll up.
    ctx.update(&mut app, keys("<ctrl+d><ctrl+d>"));

    ctx.update(
        &mut app,
        vec![
            // Scroll the mouse wheel up.
            mouse_scroll_event(0, 10, true),
        ],
    );
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn mouse_wheel_scroll_down() {
    let mut ctx = TestContext::setup_init();
    ctx.config().general.mouse_support = true;

    // Create many files to have something to scroll through
    for i in 1..=30 {
        let filename = format!("file{:02}", i);
        let content = "line 1\nline 2\nline 3\nline 4\nline 5\n".to_string();
        commit(ctx.dir.path(), &filename, &content);
        fs::write(
            ctx.dir.child(&filename),
            format!("modified content {}\n", i),
        )
        .expect("error writing to file");
    }

    let mut app = ctx.init_app();
    ctx.update(
        &mut app,
        vec![
            // Scroll the mouse wheel down.
            mouse_scroll_event(0, 10, false),
        ],
    );
    insta::assert_snapshot!(ctx.redact_buffer());
}
