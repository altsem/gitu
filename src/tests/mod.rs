use itertools::Itertools;
use std::fs;

#[macro_use]
mod helpers;
mod log;
mod rebase;

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
    commit(ctx.dir.path(), "testfile", "testing\ntesttest");
    fs::write(ctx.dir.child("testfile"), "test\ntesttest").expect("error writing to file");
    snapshot!(ctx, "jj<tab>");
}

#[test]
fn binary_file() {
    let ctx = TestContext::setup_init();
    fs::write(ctx.dir.child("binary-file"), [255]).expect("error writing to file");
    run(ctx.dir.path(), &["git", "add", "."]);
    snapshot!(ctx, "jj<tab>");
}

mod unstage {
    use super::*;

    #[test]
    fn unstage_all_staged() {
        let ctx = TestContext::setup_init();
        run(ctx.dir.path(), &["touch", "one", "two", "unaffected"]);
        run(ctx.dir.path(), &["git", "add", "one", "two"]);
        snapshot!(ctx, "jjju");
    }

    #[test]
    fn unstage_removed_line() {
        let ctx = TestContext::setup_init();
        commit(ctx.dir.path(), "firstfile", "testing\ntesttest\n");
        fs::write(ctx.dir.child("firstfile"), "weehooo\nblrergh\n").unwrap();
        run(ctx.dir.path(), &["git", "add", "."]);
        snapshot!(ctx, "jj<tab><ctrl+j><ctrl+j>u");
    }

    #[test]
    fn unstage_added_line() {
        let ctx = TestContext::setup_init();
        commit(ctx.dir.path(), "firstfile", "testing\ntesttest\n");
        fs::write(ctx.dir.child("firstfile"), "weehooo\nblrergh\n").unwrap();
        run(ctx.dir.path(), &["git", "add", "."]);
        snapshot!(ctx, "jj<tab><ctrl+j><ctrl+j><ctrl+j><ctrl+j>u");
    }
}

mod stage {
    use super::*;

    #[test]
    fn staged_file() {
        let mut ctx = TestContext::setup_init();
        run(ctx.dir.path(), &["touch", "new-file"]);
        run(ctx.dir.path(), &["git", "add", "new-file"]);

        ctx.init_state();
        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    fn stage_all_unstaged() {
        let ctx = TestContext::setup_init();
        commit(ctx.dir.path(), "firstfile", "testing\ntesttest\n");
        commit(ctx.dir.path(), "secondfile", "testing\ntesttest\n");

        fs::write(ctx.dir.child("firstfile"), "blahonga\n").unwrap();
        fs::write(ctx.dir.child("secondfile"), "blahonga\n").unwrap();
        snapshot!(ctx, "js");
    }

    #[test]
    fn stage_all_untracked() {
        let ctx = TestContext::setup_init();
        run(ctx.dir.path(), &["touch", "file-a"]);
        run(ctx.dir.path(), &["touch", "file-b"]);
        snapshot!(ctx, "js");
    }

    #[test]
    fn stage_removed_line() {
        let ctx = TestContext::setup_init();
        commit(ctx.dir.path(), "firstfile", "testing\ntesttest\n");
        fs::write(ctx.dir.child("firstfile"), "weehooo\nblrergh\n").unwrap();
        snapshot!(ctx, "jj<tab><ctrl+j><ctrl+j>s");
    }

    #[test]
    fn stage_added_line() {
        let ctx = TestContext::setup_init();
        commit(ctx.dir.path(), "firstfile", "testing\ntesttest\n");
        fs::write(ctx.dir.child("firstfile"), "weehooo\nblrergh\n").unwrap();

        snapshot!(ctx, "jj<tab><ctrl+j><ctrl+j><ctrl+j><ctrl+j>s");
    }

    #[test]
    fn stage_changes_crlf() {
        let ctx = TestContext::setup_init();
        commit(ctx.dir.path(), "testfile", "testing\r\ntesttest");
        fs::write(ctx.dir.child("testfile"), "test\r\ntesttest").expect("error writing to file");

        snapshot!(ctx, "jj<tab>");
    }
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

mod push {
    use super::*;

    #[test]
    fn push() {
        let ctx = TestContext::setup_clone();
        commit(ctx.dir.path(), "new-file", "");
        snapshot!(ctx, "Pp");
    }

    #[test]
    fn force_push() {
        let ctx = TestContext::setup_clone();
        commit(ctx.dir.path(), "new-file", "");
        snapshot!(ctx, "P-fp");
    }

    #[test]
    fn open_push_menu_after_dash_input() {
        let ctx = TestContext::setup_clone();
        commit(ctx.dir.path(), "new-file", "");
        snapshot!(ctx, "-P");
    }
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

mod stash {
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
}

mod discard {
    use super::*;

    #[test]
    pub(crate) fn discard_branch_confirm_prompt() {
        let ctx = TestContext::setup_clone();
        run(ctx.dir.path(), &["git", "branch", "asd"]);
        snapshot!(ctx, "yjK");
    }

    #[test]
    pub(crate) fn discard_branch_yes() {
        let ctx = TestContext::setup_clone();
        run(ctx.dir.path(), &["git", "branch", "asd"]);
        snapshot!(ctx, "yjKy");
    }

    #[test]
    pub(crate) fn discard_branch_no() {
        let ctx = TestContext::setup_clone();
        run(ctx.dir.path(), &["git", "branch", "asd"]);
        snapshot!(ctx, "yjKn");
    }

    #[test]
    pub(crate) fn discard_untracked_file() {
        let ctx = TestContext::setup_clone();
        run(ctx.dir.path(), &["touch", "some-file"]);
        snapshot!(ctx, "jjKy");
    }

    #[test]
    pub(crate) fn discard_untracked_staged_file() {
        let ctx = TestContext::setup_clone();
        run(ctx.dir.path(), &["touch", "some-file"]);
        run(ctx.dir.path(), &["git", "add", "some-file"]);
        snapshot!(ctx, "jsjKy");
    }

    #[test]
    pub(crate) fn discard_file_move() {
        let ctx = TestContext::setup_clone();
        commit(ctx.dir.path(), "new-file", "hello");
        run(ctx.dir.path(), &["git", "mv", "new-file", "moved-file"]);

        // TODO: Moved file is shown as 1 new and 1 deleted file.
        snapshot!(ctx, "jjKyKy");
    }

    #[test]
    pub(crate) fn discard_unstaged_delta() {
        let ctx = TestContext::setup_clone();
        commit(ctx.dir.path(), "file-one", "FOO\nBAR\n");
        fs::write(ctx.dir.child("file-one"), "blahonga\n").unwrap();
        snapshot!(ctx, "jjKy");
    }

    #[test]
    pub(crate) fn discard_unstaged_hunk() {
        let ctx = TestContext::setup_clone();
        commit(ctx.dir.path(), "file-one", "FOO\nBAR\n");
        fs::write(ctx.dir.child("file-one"), "blahonga\n").unwrap();
        snapshot!(ctx, "jj<tab>jKy");
    }

    #[test]
    pub(crate) fn discard_staged_file() {
        let ctx = TestContext::setup_clone();
        commit(ctx.dir.path(), "file-one", "FOO\nBAR\n");
        fs::write(ctx.dir.child("file-one"), "blahonga\n").unwrap();
        run(ctx.dir.path(), &["git", "add", "."]);
        snapshot!(ctx, "jjKy");
    }

    // FIXME Deleting branches doesn't work with the test-setup
    // #[test]
    // fn discard_branch() {
    //     let mut ctx = TestContext::setup_clone();
    //     let mut state = ctx.init_state();
    //     state
    //         .update(&mut ctx.term, &keys("yjKy"))
    //         .unwrap();
    //     insta::assert_snapshot!(ctx.redact_buffer());
    // }
}

mod reset {
    use super::*;

    fn setup() -> TestContext {
        let ctx = TestContext::setup_clone();
        commit(ctx.dir.path(), "unwanted-file", "");
        ctx
    }

    #[test]
    pub(crate) fn reset_menu() {
        snapshot!(setup(), "lljX");
    }

    #[test]
    pub(crate) fn reset_soft_prompt() {
        snapshot!(setup(), "lljXsq");
    }

    #[test]
    pub(crate) fn reset_soft() {
        snapshot!(setup(), "lljXs<enter>q");
    }

    #[test]
    pub(crate) fn reset_mixed() {
        snapshot!(setup(), "lljXm<enter>q");
    }

    #[test]
    fn reset_hard() {
        snapshot!(setup(), "lljXh<enter>q");
    }
}

#[test]
fn show_refs() {
    let ctx = TestContext::setup_clone();
    run(ctx.dir.path(), &["git", "tag", "same-name"]);
    run(ctx.dir.path(), &["git", "checkout", "-b", "same-name"]);
    snapshot!(ctx, "y");
}

mod checkout {
    use super::*;

    #[test]
    pub(crate) fn checkout_menu() {
        let ctx = TestContext::setup_clone();
        run(ctx.dir.path(), &["git", "branch", "other-branch"]);
        snapshot!(ctx, "yjb");
    }

    #[test]
    pub(crate) fn switch_branch_selected() {
        let ctx = TestContext::setup_clone();
        run(ctx.dir.path(), &["git", "branch", "other-branch"]);
        snapshot!(ctx, "yjjbb<enter>");
    }

    #[test]
    pub(crate) fn switch_branch_input() {
        let ctx = TestContext::setup_clone();
        run(ctx.dir.path(), &["git", "branch", "hi"]);
        snapshot!(ctx, "yjjbbhi<enter>");
    }

    #[test]
    pub(crate) fn checkout_new_branch() {
        snapshot!(TestContext::setup_clone(), "bcf<esc>bcx<enter>");
    }
}

#[test]
fn updated_externally() {
    let mut ctx = TestContext::setup_init();
    fs::write(ctx.dir.child("b"), "test").unwrap();

    let mut state = ctx.init_state();
    state.update(&mut ctx.term, &keys("jjsj")).unwrap();

    fs::write(ctx.dir.child("a"), "test").unwrap();

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

mod scroll {
    use super::*;

    fn setup_scroll() -> (TestContext, crate::state::State) {
        let mut ctx = TestContext::setup_init();
        for file in ["file-1", "file-2", "file-3"] {
            commit(ctx.dir.path(), file, "");
            fs::write(
                ctx.dir.child(file),
                (1..=20)
                    .map(|i| format!("line {} ({})", i, file))
                    .join("\n"),
            )
            .unwrap();
        }

        let mut state = ctx.init_state();
        state
            .update(&mut ctx.term, &keys("jjjj<tab>k<tab>k<tab>"))
            .unwrap();
        (ctx, state)
    }

    #[test]
    fn scroll_down() {
        let (mut ctx, mut state) = setup_scroll();

        state.update(&mut ctx.term, &keys("<ctrl+d>")).unwrap();

        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    fn scroll_past_selection() {
        let (mut ctx, mut state) = setup_scroll();

        state
            .update(&mut ctx.term, &keys("<ctrl+d><ctrl+d><ctrl+d>"))
            .unwrap();

        insta::assert_snapshot!(ctx.redact_buffer());
    }
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

mod quit {
    use super::*;

    #[test]
    pub(crate) fn quit() {
        let state = snapshot!(TestContext::setup_init(), "q");
        assert!(state.quit);
    }

    #[test]
    pub(crate) fn quit_from_menu() {
        let state = snapshot!(TestContext::setup_init(), "hq");
        assert!(!state.quit);
    }

    #[test]
    pub(crate) fn confirm_quit_prompt() {
        let mut ctx = TestContext::setup_init();
        ctx.config().general.confirm_quit.enabled = true;

        let state = snapshot!(ctx, "q");
        assert!(!state.quit);
    }

    #[test]
    pub(crate) fn confirm_quit() {
        let mut ctx = TestContext::setup_init();
        ctx.config().general.confirm_quit.enabled = true;

        let state = snapshot!(ctx, "qy");
        assert!(state.quit);
    }
}
