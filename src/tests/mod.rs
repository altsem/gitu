use std::fs;

#[macro_use]
mod helpers;
mod arg;
mod discard;
mod editor;
mod fetch;
mod log;
mod pull;
mod push;
mod quit;
mod rebase;
mod reset;
mod stage;
mod stash;
mod unstage;

use helpers::{clone_and_commit, commit, keys, run, TestContext};

#[test]
fn no_repo() {
    let mut ctx = TestContext::setup_init();

    ctx.init_state();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn help_menu() {
    let mut ctx = TestContext::setup_init();

    let mut state = ctx.init_state();
    state.update(&mut ctx.term, &keys("h")).unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn fresh_init() {
    let mut ctx = TestContext::setup_init();

    ctx.init_state();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn new_file() {
    let mut ctx = TestContext::setup_init();
    run(ctx.dir.path(), &["touch", "new-file"]);

    ctx.init_state();
    insta::assert_snapshot!(ctx.redact_buffer());
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
    fs::write(ctx.dir.child("binary-file"), [255]).expect("error writing to file");
    run(ctx.dir.path(), &["git", "add", "."]);
    snapshot!(ctx, "jj<tab>");
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
fn rebase_conflict() {
    let mut ctx = TestContext::setup_clone();
    commit(ctx.dir.path(), "new-file", "hello");

    run(ctx.dir.path(), &["git", "checkout", "-b", "other-branch"]);
    commit(ctx.dir.path(), "new-file", "hey");

    run(ctx.dir.path(), &["git", "checkout", "main"]);
    commit(ctx.dir.path(), "new-file", "hi");

    run(ctx.dir.path(), &["git", "checkout", "other-branch"]);
    run(ctx.dir.path(), &["git", "rebase", "main"]);

    ctx.init_state();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn merge_conflict() {
    let mut ctx = TestContext::setup_clone();
    commit(ctx.dir.path(), "new-file", "hello");

    run(ctx.dir.path(), &["git", "checkout", "-b", "other-branch"]);
    commit(ctx.dir.path(), "new-file", "hey");

    run(ctx.dir.path(), &["git", "checkout", "main"]);
    commit(ctx.dir.path(), "new-file", "hi");

    run(ctx.dir.path(), &["git", "merge", "other-branch"]);

    ctx.init_state();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn revert_conflict() {
    let mut ctx = TestContext::setup_clone();
    commit(ctx.dir.path(), "new-file", "hey");
    commit(ctx.dir.path(), "new-file", "hi");

    run(ctx.dir.path(), &["git", "revert", "HEAD~1"]);

    ctx.init_state();
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
    snapshot!(ctx, "llV-eV");
}

#[test]
fn revert_commit() {
    let ctx = TestContext::setup_clone();
    snapshot!(ctx, "llV-eV<enter>");
}

#[test]
fn moved_file() {
    let mut ctx = TestContext::setup_clone();
    commit(ctx.dir.path(), "new-file", "hello");
    run(ctx.dir.path(), &["git", "mv", "new-file", "moved-file"]);

    ctx.init_state();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn hide_untracked() {
    let mut ctx = TestContext::setup_clone();
    run(ctx.dir.path(), &["touch", "i-am-untracked"]);

    let mut state = ctx.init_state();
    let mut config = state.repo.config().unwrap();
    config.set_str("status.showUntrackedFiles", "off").unwrap();

    state.update(&mut ctx.term, &keys("g")).unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn new_commit() {
    let mut ctx = TestContext::setup_clone();
    commit(ctx.dir.path(), "new-file", "");

    ctx.init_state();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn fetch_all() {
    let ctx = TestContext::setup_clone();
    clone_and_commit(&ctx.remote_dir, "remote-file", "hello");
    snapshot!(ctx, "fa");
}

#[test]
fn pull() {
    let ctx = TestContext::setup_clone();
    clone_and_commit(&ctx.remote_dir, "remote-file", "hello");
    snapshot!(ctx, "Fp");
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

mod checkout {
    use super::*;

    #[test]
    pub(crate) fn checkout_menu() {
        let ctx = TestContext::setup_clone();
        run(ctx.dir.path(), &["git", "branch", "other-branch"]);
        snapshot!(ctx, "Yjb");
    }

    #[test]
    pub(crate) fn switch_branch_selected() {
        let ctx = TestContext::setup_clone();
        run(ctx.dir.path(), &["git", "branch", "other-branch"]);
        snapshot!(ctx, "Yjjbb<enter>");
    }

    #[test]
    pub(crate) fn switch_branch_input() {
        let ctx = TestContext::setup_clone();
        run(ctx.dir.path(), &["git", "branch", "hi"]);
        snapshot!(ctx, "Yjjbbhi<enter>");
    }

    #[test]
    pub(crate) fn checkout_new_branch() {
        snapshot!(TestContext::setup_clone(), "bcf<esc>bcx<enter>");
    }
}

#[test]
fn updated_externally() {
    let mut ctx = TestContext::setup_init();
    fs::write(ctx.dir.child("b"), "test\n").unwrap();

    let mut state = ctx.init_state();
    state.update(&mut ctx.term, &keys("jjsj")).unwrap();

    fs::write(ctx.dir.child("a"), "test\n").unwrap();

    state.update(&mut ctx.term, &keys("g")).unwrap();
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

    let _state = ctx.init_state_at_path(ctx.dir.child("test-submodule"));
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
    let mut state = ctx.init_state();
    state
        .repo
        .config()
        .unwrap()
        .set_bool("core.autocrlf", true)
        .unwrap();

    commit(ctx.dir.path(), "crlf.txt", "unchanged\r\nunchanged\r\n");
    fs::write(ctx.dir.child("crlf.txt"), "unchanged\r\nchanged\r\n").unwrap();
    state.update(&mut ctx.term, &keys("g")).unwrap();

    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn tab_diff() {
    let mut ctx = TestContext::setup_init();
    let mut state = ctx.init_state();

    commit(ctx.dir.path(), "tab.txt", "this has no tab prefixed\n");
    fs::write(ctx.dir.child("tab.txt"), "\tthis has a tab prefixed\n").unwrap();
    state.update(&mut ctx.term, &keys("g")).unwrap();

    insta::assert_snapshot!(ctx.redact_buffer());
}
