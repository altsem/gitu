//! This module contains integration tests for Gitu.
//! Each test:
//! - sets up a temporary git repository in a temporary directory
//! - runs some commands
//! - asserts the output (`cargo insta` is used a lot https://insta.rs)
//! - cleans up the temporary directory
//!
//! Each test typically sets up its own git repo under `testfiles/`
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
use stdext::function_name;
use url::Url;

use crate::tests::helpers::run_ignore_status;

#[test]
fn help_menu() {
    let mut ctx = setup_clone!();

    let mut app = ctx.init_app();
    ctx.update(&mut app, keys("h"));
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn fresh_init() {
    let mut ctx = setup_clone!();
    run(&ctx.dir, &["rm", "-rf", ".git"]);
    run(&ctx.dir, &["rm", "initial-file"]);
    run(&ctx.dir, &["git", "init", "--initial-branch=main"]);

    ctx.init_app();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn new_file() {
    let mut ctx = setup_clone!();
    run(&ctx.dir, &["touch", "new-file"]);

    ctx.init_app();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn deleted_file() {
    let ctx = setup_clone!();
    commit(&ctx.dir, "new-file", "testing\ntesttest\n");
    run(&ctx.dir, &["rm", "new-file"]);
    snapshot!(ctx, "");
}

#[test]
fn copied_file() {
    let ctx = setup_clone!();
    commit(&ctx.dir, "new-file", "testing\ntesttest\n");
    run(&ctx.dir, &["cp", "new-file", "copied-file"]);
    run(&ctx.dir, &["git", "add", "-N", "."]);
    snapshot!(ctx, "");
}

#[test]
fn unstaged_changes() {
    let ctx = setup_clone!();
    commit(&ctx.dir, "testfile", "testing\ntesttest\n");
    fs::write(ctx.dir.join("testfile"), "test\ntesttest\n").expect("error writing to file");
    snapshot!(ctx, "jj<tab>");
}

#[test]
fn binary_file() {
    let ctx = setup_clone!();
    fs::write(ctx.dir.join("binary-file"), [0, 255]).expect("error writing to file");
    run(&ctx.dir, &["git", "add", "."]);
    snapshot!(ctx, "jj<tab>");
}

#[test]
fn non_ascii_filename() {
    let ctx = setup_clone!();
    commit(&ctx.dir, "höhöhö", "hehehe\n");
    fs::write(ctx.dir.join("höhöhö"), "hahaha\n").expect("error writing to file");
    snapshot!(ctx, "jj<tab>");
}

#[test]
fn collapsed_sections_config() {
    let mut ctx = setup_clone!();
    ctx.config().general.collapsed_sections = vec![
        "untracked".into(),
        "recent_commits".into(),
        "branch_status".into(),
        // TODO rebase / revert/ merge conlict?
    ];
    fs::write(ctx.dir.join("untracked_file.txt"), "").unwrap();

    snapshot!(ctx, "");
}

#[test]
fn stash_list_with_limit() {
    let mut ctx = setup_clone!();
    ctx.config().general.stash_list_limit = 2;

    fs::write(ctx.dir.join("file1.txt"), "content").unwrap();
    run(&ctx.dir, &["git", "add", "."]);
    run(&ctx.dir, &["git", "stash", "save", "firststash"]);
    fs::write(ctx.dir.join("file2.txt"), "content").unwrap();
    run(&ctx.dir, &["git", "add", "."]);
    run(&ctx.dir, &["git", "stash", "save", "secondstash"]);
    fs::write(ctx.dir.join("file3.txt"), "content").unwrap();
    run(&ctx.dir, &["git", "add", "."]);
    run(&ctx.dir, &["git", "stash", "save", "thirdstash"]);

    snapshot!(ctx, "");
}

#[test]
fn recent_commits_with_limit() {
    let mut ctx = setup_clone!();
    ctx.config().general.recent_commits_limit = 2;
    commit(&ctx.dir, "firstfile", "testing\ntesttest\n");
    commit(&ctx.dir, "secondfile", "testing\ntesttest\n");
    commit(&ctx.dir, "thirdfile", "testing\ntesttest\n");
    snapshot!(ctx, "");
}

#[test]
fn log() {
    let ctx = setup_clone!();
    commit(&ctx.dir, "firstfile", "testing\ntesttest\n");
    run(&ctx.dir, &["git", "tag", "-am", ".", "annotated"]);
    commit(&ctx.dir, "secondfile", "testing\ntesttest\n");
    run(&ctx.dir, &["git", "tag", "a-tag"]);
    snapshot!(ctx, "ll");
}

#[test]
fn show() {
    let ctx = setup_clone!();
    commit(&ctx.dir, "firstfile", "This should be visible\n");
    snapshot!(ctx, "ll<enter>");
}

#[test]
fn show_stash() {
    let ctx = setup_clone!();

    // Unstaged changes should be shown as a diff against a tracked file.
    commit(&ctx.dir, "unstaged.txt", "");

    // Staged changes
    fs::write(ctx.dir.join("staged.txt"), "staged\n").unwrap();
    run(&ctx.dir, &["git", "add", "staged.txt"]);

    // Unstaged changes
    fs::write(ctx.dir.join("unstaged.txt"), "unstaged\n").unwrap();

    // Untracked changes
    fs::write(ctx.dir.join("untracked.txt"), "untracked\n").unwrap();

    run(
        &ctx.dir,
        &[
            "git",
            "stash",
            "push",
            "--include-untracked",
            "--message",
            "firststash",
        ],
    );

    snapshot!(ctx, "jj<enter>");
}

#[test]
fn rebase_conflict() {
    let mut ctx = setup_clone!();
    commit(&ctx.dir, "new-file", "hello");

    run(&ctx.dir, &["git", "checkout", "-b", "other-branch"]);
    commit(&ctx.dir, "new-file", "hey");

    run(&ctx.dir, &["git", "checkout", "main"]);
    commit(&ctx.dir, "new-file", "hi");

    run(&ctx.dir, &["git", "checkout", "other-branch"]);
    run_ignore_status(&ctx.dir, &["git", "rebase", "main"]);

    ctx.init_app();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn merge_conflict() {
    let mut ctx = setup_clone!();
    commit(&ctx.dir, "new-file", "hello");
    commit(&ctx.dir, "new-file-2", "hello");

    run(&ctx.dir, &["git", "checkout", "-b", "other-branch"]);
    commit(&ctx.dir, "new-file", "hey");
    commit(&ctx.dir, "new-file-2", "hey");

    run(&ctx.dir, &["git", "checkout", "main"]);
    commit(&ctx.dir, "new-file", "hi");
    commit(&ctx.dir, "new-file-2", "hi");

    run_ignore_status(&ctx.dir, &["git", "merge", "other-branch"]);

    ctx.init_app();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn revert_conflict() {
    let mut ctx = setup_clone!();
    commit(&ctx.dir, "new-file", "hey");
    commit(&ctx.dir, "new-file", "hi");

    run_ignore_status(&ctx.dir, &["git", "revert", "HEAD~1"]);

    ctx.init_app();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn revert_abort() {
    let ctx = setup_clone!();
    commit(&ctx.dir, "new-file", "hey");
    commit(&ctx.dir, "new-file", "hi");

    run_ignore_status(&ctx.dir, &["git", "revert", "HEAD~1"]);

    snapshot!(ctx, "Va");
}

#[test]
fn revert_menu() {
    let ctx = setup_clone!();
    snapshot!(ctx, "llV");
}

#[test]
fn revert_commit_prompt() {
    let ctx = setup_clone!();
    snapshot!(ctx, "llVV");
}

#[test]
fn revert_commit() {
    let ctx = setup_clone!();
    snapshot!(ctx, "llV-EV<enter>");
}

#[test]
fn moved_file() {
    let mut ctx = setup_clone!();
    commit(&ctx.dir, "new-file", "hello");
    run(&ctx.dir, &["git", "mv", "new-file", "moved-file"]);

    ctx.init_app();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
#[cfg(not(target_os = "windows"))]
fn chmod_file() {
    let mut ctx = setup_clone!();
    commit(&ctx.dir, "test-file", "hello\nworld\n");
    run(&ctx.dir, &["chmod", "+x", "test-file"]);

    ctx.init_app();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn hide_untracked() {
    let mut ctx = setup_clone!();
    run(&ctx.dir, &["touch", "i-am-untracked"]);

    let mut app = ctx.init_app();
    let mut config = app.state.repo.config().unwrap();
    // Git expects "no|normal|all" here; "off" can error on some versions and break `git status`.
    config.set_str("status.showUntrackedFiles", "no").unwrap();

    ctx.update(&mut app, keys("g"));
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn new_commit() {
    let mut ctx = setup_clone!();
    commit(&ctx.dir, "new-file", "");

    ctx.init_app();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn fetch_all() {
    let ctx = setup_clone!();
    clone_and_commit(&ctx.remote_dir, "remote-file", "hello");
    snapshot!(ctx, "fa");
}

mod show_refs {
    use super::*;

    #[test]
    fn show_refs_at_local_branch() {
        let ctx = setup_clone!();
        run(&ctx.dir, &["git", "tag", "main"]);
        snapshot!(ctx, "Y");
    }

    #[test]
    fn show_refs_at_remote_branch() {
        let ctx = setup_clone!();
        snapshot!(ctx, "Yjjjjbb<enter>Y");
    }

    #[test]
    fn show_refs_at_tag() {
        let ctx = setup_clone!();
        run(&ctx.dir, &["git", "tag", "v1.0"]);
        snapshot!(ctx, "Yjjjjjjbb<enter>Y");
    }
}

#[test]
fn updated_externally() {
    let mut ctx = setup_clone!();
    fs::write(ctx.dir.join("b"), "test\n").unwrap();

    let mut app = ctx.init_app();
    ctx.update(&mut app, keys("jjsj"));

    fs::write(ctx.dir.join("a"), "test\n").unwrap();

    ctx.update(&mut app, keys("g"));
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn stage_last_hunk_of_first_delta() {
    let ctx = setup_clone!();
    commit(&ctx.dir, "file-one", "asdf\nblahonga\n");
    commit(&ctx.dir, "file-two", "FOO\nBAR\n");
    fs::write(ctx.dir.join("file-one"), "blahonga\n").unwrap();
    fs::write(ctx.dir.join("file-two"), "blahonga\n").unwrap();

    snapshot!(ctx, "jj<tab>js");
}

#[test]
fn go_down_past_collapsed() {
    let ctx = setup_clone!();
    commit(&ctx.dir, "file-one", "asdf\nblahonga\n");
    commit(&ctx.dir, "file-two", "FOO\nBAR\n");
    fs::write(ctx.dir.join("file-one"), "blahonga\n").unwrap();
    fs::write(ctx.dir.join("file-two"), "blahonga\n").unwrap();

    snapshot!(ctx, "jjj");
}

#[test]
fn inside_submodule() {
    let mut ctx = setup_clone!();
    let url = Url::from_file_path(ctx.remote_dir.as_path()).unwrap();
    run(
        &ctx.dir,
        &[
            "git",
            "-c",
            "protocol.file.allow=always",
            "submodule",
            "add",
            url.as_str(),
            "test-submodule",
        ],
    );

    let _app = ctx.init_app_at_path(ctx.dir.join("test-submodule"));
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn syntax_highlighted() {
    let ctx = setup_clone!();
    commit(
        &ctx.dir,
        "syntax-highlighted.rs",
        "fn main() {\n    println!(\"Hey\");\n}\n",
    );
    fs::write(
        ctx.dir.join("syntax-highlighted.rs"),
        "fn main() {\n    println!(\"Bye\");\n}\n",
    )
    .unwrap();

    snapshot!(ctx, "jj<tab>");
}

#[test]
fn crlf_diff() {
    let mut ctx = setup_clone!();
    let mut app = ctx.init_app();
    app.state
        .repo
        .config()
        .unwrap()
        .set_bool("core.autocrlf", true)
        .unwrap();

    commit(&ctx.dir, "crlf.txt", "unchanged\r\nunchanged\r\n");
    fs::write(ctx.dir.join("crlf.txt"), "unchanged\r\nchanged\r\n").unwrap();
    ctx.update(&mut app, keys("g"));

    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn tab_diff() {
    let mut ctx = setup_clone!();
    let mut app = ctx.init_app();

    commit(&ctx.dir, "tab.txt", "this has no tab prefixed\n");
    fs::write(ctx.dir.join("tab.txt"), "\tthis has a tab prefixed\n").unwrap();
    ctx.update(&mut app, keys("g"));

    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn non_utf8_diff() {
    let mut ctx = setup_clone!();
    let mut app = ctx.init_app();

    commit(&ctx.dir, "non_utf8.txt", "File with valid UTF-8");
    fs::write(
        ctx.dir.join("non_utf8.txt"),
        b"File with invalid UTF-8: \xff\xfe\n",
    )
    .unwrap();
    ctx.update(&mut app, keys("g"));

    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn ext_diff() {
    let mut ctx = setup_clone!();
    let mut app = ctx.init_app();

    fs::write(ctx.dir.join("unstaged.txt"), "unstaged\n").unwrap();
    fs::write(ctx.dir.join("staged.txt"), "staged\n").unwrap();
    run(&ctx.dir, &["git", "add", "-N", "unstaged.txt"]);
    run(&ctx.dir, &["git", "add", "staged.txt"]);
    run(&ctx.dir, &["git", "config", "diff.external", "/dev/null"]);
    ctx.update(&mut app, keys("g"));

    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn mouse_select_item() {
    let mut ctx = setup_clone!();
    ctx.config().general.mouse_support = true;

    commit(&ctx.dir, "testfile", "testing\ntesttest\n");

    let mut app = ctx.init_app();
    ctx.update(&mut app, vec![mouse_event(0, 5, MouseButton::Left)]);
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn mouse_select_hunk_line() {
    let mut ctx = setup_clone!();
    ctx.config().general.mouse_support = true;

    fs::write(ctx.dir.join("testfile"), "test\ntesttest\n").expect("error writing to file");
    run(&ctx.dir, &["git", "add", "."]);
    fs::write(ctx.dir.join("testfile"), "test\nmoretest\n").expect("error writing to file");

    let mut app = ctx.init_app();
    ctx.update(
        &mut app,
        vec![
            // Select the file with an unstaged change.
            mouse_event(0, 5, MouseButton::Left),
            // Expand the selected file.
            mouse_event(0, 5, MouseButton::Left),
            // Select the hunk line.
            mouse_event(0, 9, MouseButton::Left),
        ],
    );
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn mouse_select_ignore_empty_lines() {
    let mut ctx = setup_clone!();
    ctx.config().general.mouse_support = true;

    commit(&ctx.dir, "testfile", "testing\ntesttest\n");
    fs::write(ctx.dir.join("testfile"), "test\nmoretest\n").expect("error writing to file");

    let mut app = ctx.init_app();
    ctx.update(
        &mut app,
        vec![
            // Click the last unstaged change.
            mouse_event(0, 5, MouseButton::Left),
            // Click the space underneath the last unstaged change.
            mouse_event(0, 6, MouseButton::Left),
        ],
    );
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn mouse_select_ignore_empty_region() {
    let mut ctx = setup_clone!();
    ctx.config().general.mouse_support = true;

    commit(&ctx.dir, "testfile", "testing\ntesttest\n");
    fs::write(ctx.dir.join("testfile"), "test\nmoretest\n").expect("error writing to file");

    let mut app = ctx.init_app();
    ctx.update(
        &mut app,
        vec![
            // Click the last unstaged change.
            mouse_event(0, 5, MouseButton::Left),
            // Click the open space at the bottom of the screen.
            mouse_event(0, 10, MouseButton::Left),
        ],
    );
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn mouse_toggle_selected_item() {
    let mut ctx = setup_clone!();
    ctx.config().general.mouse_support = true;

    commit(&ctx.dir, "testfile", "testing\ntesttest\n");
    fs::write(ctx.dir.join("testfile"), "test\nmoretest\n").expect("error writing to file");

    let mut app = ctx.init_app();
    ctx.update(
        &mut app,
        vec![
            // Click the last unstaged change.
            mouse_event(0, 5, MouseButton::Left),
            // Click the last unstaged change.
            mouse_event(0, 5, MouseButton::Left),
        ],
    );
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn mouse_show_item() {
    let mut ctx = setup_clone!();
    ctx.config().general.mouse_support = true;

    commit(&ctx.dir, "testfile", "testing\ntesttest\n");

    let mut app = ctx.init_app();
    ctx.update(
        &mut app,
        vec![
            // Right-click the last unstaged change.
            mouse_event(0, 5, MouseButton::Right),
        ],
    );
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn mouse_show_ignore_empty_lines() {
    let mut ctx = setup_clone!();
    ctx.config().general.mouse_support = true;

    commit(&ctx.dir, "testfile", "testing\ntesttest\n");
    fs::write(ctx.dir.join("testfile"), "test\nmoretest\n").expect("error writing to file");
    run(&ctx.dir, &["git", "add", "."]);
    run(&ctx.dir, &["git", "stash", "save", "firststash"]);

    let mut app = ctx.init_app();
    ctx.update(
        &mut app,
        vec![
            // Left-click the last stash.
            mouse_event(0, 5, MouseButton::Left),
            // Right-click the space underneath the last stash.
            mouse_event(0, 6, MouseButton::Right),
        ],
    );
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn mouse_show_ignore_empty_region() {
    let mut ctx = setup_clone!();
    ctx.config().general.mouse_support = true;

    commit(&ctx.dir, "testfile", "testing\ntesttest\n");
    fs::write(ctx.dir.join("testfile"), "test\nmoretest\n").expect("error writing to file");
    run(&ctx.dir, &["git", "add", "."]);
    run(&ctx.dir, &["git", "stash", "save", "firststash"]);

    let mut app = ctx.init_app();
    ctx.update(
        &mut app,
        vec![
            // Left-click the last stash change.
            mouse_event(0, 5, MouseButton::Left),
            // Right-click the open space at the bottom of the screen.
            mouse_event(0, 10, MouseButton::Right),
        ],
    );
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn mouse_wheel_scroll_up() {
    let mut ctx = setup_clone!();
    ctx.config().general.mouse_support = true;

    // Create many files to have something to scroll through
    for i in 1..=30 {
        let filename = format!("file{:02}", i);
        let content = "line 1\nline 2\nline 3\nline 4\nline 5\n".to_string();
        commit(&ctx.dir, &filename, &content);
        fs::write(ctx.dir.join(&filename), format!("modified content {}\n", i))
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
    let mut ctx = setup_clone!();
    ctx.config().general.mouse_support = true;

    // Create many files to have something to scroll through
    for i in 1..=30 {
        let filename = format!("file{:02}", i);
        let content = "line 1\nline 2\nline 3\nline 4\nline 5\n".to_string();
        commit(&ctx.dir, &filename, &content);
        fs::write(ctx.dir.join(&filename), format!("modified content {}\n", i))
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
