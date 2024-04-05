use itertools::Itertools;
use std::fs;

mod helpers;
mod log;
mod rebase;

use helpers::{clone_and_commit, commit, keys, run, TestContext};

#[test]
fn no_repo() {
    let mut ctx = TestContext::setup_init(80, 20);

    ctx.init_state();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn help_menu() {
    let mut ctx = TestContext::setup_init(80, 20);

    let mut state = ctx.init_state();
    state.update(&mut ctx.term, &keys("h")).unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn fresh_init() {
    let mut ctx = TestContext::setup_init(80, 20);

    ctx.init_state();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn new_file() {
    let mut ctx = TestContext::setup_init(80, 20);
    run(ctx.dir.path(), &["touch", "new-file"]);

    ctx.init_state();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn unstaged_changes() {
    let mut ctx = TestContext::setup_init(80, 20);
    commit(ctx.dir.path(), "testfile", "testing\ntesttest");
    fs::write(ctx.dir.child("testfile"), "test\ntesttest").expect("error writing to file");

    let mut state = ctx.init_state();
    state.update(&mut ctx.term, &keys("jj<tab>")).unwrap();

    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn binary_file() {
    let mut ctx = TestContext::setup_init(80, 20);
    fs::write(ctx.dir.child("binary-file"), [255]).expect("error writing to file");
    run(ctx.dir.path(), &["git", "add", "."]);

    let mut state = ctx.init_state();
    state.update(&mut ctx.term, &keys("jj<tab>")).unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
}

mod unstage {
    use super::*;

    #[test]
    fn unstage_all_staged() {
        let mut ctx = TestContext::setup_init(80, 20);
        run(ctx.dir.path(), &["touch", "one", "two", "unaffected"]);
        run(ctx.dir.path(), &["git", "add", "one", "two"]);

        let mut state = ctx.init_state();
        state.update(&mut ctx.term, &keys("jjju")).unwrap();

        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    fn unstage_removed_line() {
        let mut ctx = TestContext::setup_init(80, 20);
        commit(ctx.dir.path(), "firstfile", "testing\ntesttest\n");
        fs::write(ctx.dir.child("firstfile"), "weehooo\nblrergh\n").unwrap();
        run(ctx.dir.path(), &["git", "add", "."]);

        let mut state = ctx.init_state();
        state
            .update(&mut ctx.term, &keys("jj<tab><ctrl+j><ctrl+j>u"))
            .unwrap();

        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    fn unstage_added_line() {
        let mut ctx = TestContext::setup_init(80, 20);
        commit(ctx.dir.path(), "firstfile", "testing\ntesttest\n");
        fs::write(ctx.dir.child("firstfile"), "weehooo\nblrergh\n").unwrap();
        run(ctx.dir.path(), &["git", "add", "."]);

        let mut state = ctx.init_state();
        state
            .update(
                &mut ctx.term,
                &keys("jj<tab><ctrl+j><ctrl+j><ctrl+j><ctrl+j>u"),
            )
            .unwrap();

        insta::assert_snapshot!(ctx.redact_buffer());
    }
}

mod stage {
    use super::*;

    #[test]
    fn staged_file() {
        let mut ctx = TestContext::setup_init(80, 20);
        run(ctx.dir.path(), &["touch", "new-file"]);
        run(ctx.dir.path(), &["git", "add", "new-file"]);

        ctx.init_state();
        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    fn stage_all_unstaged() {
        let mut ctx = TestContext::setup_init(80, 20);
        commit(ctx.dir.path(), "firstfile", "testing\ntesttest\n");
        commit(ctx.dir.path(), "secondfile", "testing\ntesttest\n");

        fs::write(ctx.dir.child("firstfile"), "blahonga\n").unwrap();
        fs::write(ctx.dir.child("secondfile"), "blahonga\n").unwrap();

        let mut state = ctx.init_state();
        state.update(&mut ctx.term, &keys("js")).unwrap();

        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    fn stage_all_untracked() {
        let mut ctx = TestContext::setup_init(80, 20);
        run(ctx.dir.path(), &["touch", "file-a"]);
        run(ctx.dir.path(), &["touch", "file-b"]);

        let mut state = ctx.init_state();
        state.update(&mut ctx.term, &keys("js")).unwrap();

        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    fn stage_removed_line() {
        let mut ctx = TestContext::setup_init(80, 20);
        commit(ctx.dir.path(), "firstfile", "testing\ntesttest\n");
        fs::write(ctx.dir.child("firstfile"), "weehooo\nblrergh\n").unwrap();

        let mut state = ctx.init_state();
        state
            .update(&mut ctx.term, &keys("jj<tab><ctrl+j><ctrl+j>s"))
            .unwrap();

        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    fn stage_added_line() {
        let mut ctx = TestContext::setup_init(80, 20);
        commit(ctx.dir.path(), "firstfile", "testing\ntesttest\n");
        fs::write(ctx.dir.child("firstfile"), "weehooo\nblrergh\n").unwrap();

        let mut state = ctx.init_state();
        state
            .update(
                &mut ctx.term,
                &keys("jj<tab><ctrl+j><ctrl+j><ctrl+j><ctrl+j>s"),
            )
            .unwrap();

        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    fn stage_changes_crlf() {
        let mut ctx = TestContext::setup_init(80, 20);
        commit(ctx.dir.path(), "testfile", "testing\r\ntesttest");
        fs::write(ctx.dir.child("testfile"), "test\r\ntesttest").expect("error writing to file");

        let mut state = ctx.init_state();
        state.update(&mut ctx.term, &keys("jj<tab>")).unwrap();

        insta::assert_snapshot!(ctx.redact_buffer());
    }
}

#[test]
fn log() {
    let mut ctx = TestContext::setup_clone(80, 20);
    commit(ctx.dir.path(), "firstfile", "testing\ntesttest\n");
    run(ctx.dir.path(), &["git", "tag", "-am", ".", "annotated"]);
    commit(ctx.dir.path(), "secondfile", "testing\ntesttest\n");
    run(ctx.dir.path(), &["git", "tag", "a-tag"]);

    let mut state = ctx.init_state();
    state.update(&mut ctx.term, &keys("ll")).unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn show() {
    let mut ctx = TestContext::setup_clone(80, 20);
    commit(ctx.dir.path(), "firstfile", "This should be visible\n");

    let mut state = ctx.init_state();
    state.update(&mut ctx.term, &keys("ll<enter>")).unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn rebase_conflict() {
    let mut ctx = TestContext::setup_clone(80, 20);
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
    let mut ctx = TestContext::setup_clone(80, 20);
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
    let mut ctx = TestContext::setup_clone(80, 20);
    commit(ctx.dir.path(), "new-file", "hello");
    run(ctx.dir.path(), &["git", "mv", "new-file", "moved-file"]);

    ctx.init_state();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn hide_untracked() {
    let mut ctx = TestContext::setup_clone(80, 10);
    run(ctx.dir.path(), &["touch", "i-am-untracked"]);

    let mut state = ctx.init_state();
    let mut config = state.repo.config().unwrap();
    config.set_str("status.showUntrackedFiles", "off").unwrap();
    state.update(&mut ctx.term, &keys("g")).unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn new_commit() {
    let mut ctx = TestContext::setup_clone(80, 10);
    commit(ctx.dir.path(), "new-file", "");

    ctx.init_state();
    insta::assert_snapshot!(ctx.redact_buffer());
}

mod push {
    use super::*;

    #[test]
    fn push() {
        let mut ctx = TestContext::setup_clone(80, 10);
        commit(ctx.dir.path(), "new-file", "");

        let mut state = ctx.init_state();
        state.update(&mut ctx.term, &keys("Pp")).unwrap();
        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    fn force_push() {
        let mut ctx = TestContext::setup_clone(80, 10);
        commit(ctx.dir.path(), "new-file", "");

        let mut state = ctx.init_state();
        state.update(&mut ctx.term, &keys("P-fp")).unwrap();
        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    fn open_push_menu_after_dash_input() {
        let mut ctx = TestContext::setup_clone(80, 10);
        commit(ctx.dir.path(), "new-file", "");

        let mut state = ctx.init_state();
        state.update(&mut ctx.term, &keys("-P")).unwrap();
        insta::assert_snapshot!(ctx.redact_buffer());
    }
}

#[test]
fn fetch_all() {
    let mut ctx = TestContext::setup_clone(80, 10);
    clone_and_commit(&ctx.remote_dir, "remote-file", "hello");

    let mut state = ctx.init_state();
    state.update(&mut ctx.term, &keys("fa")).unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn pull() {
    let mut ctx = TestContext::setup_clone(80, 10);
    clone_and_commit(&ctx.remote_dir, "remote-file", "hello");

    let mut state = ctx.init_state();
    state.update(&mut ctx.term, &keys("Fp")).unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
}

mod stash {
    use super::*;
    use crate::state::State;

    fn setup() -> (TestContext, State) {
        let mut ctx = TestContext::setup_clone(80, 20);
        fs::write(ctx.dir.child("file-one"), "blahonga\n").unwrap();
        fs::write(ctx.dir.child("file-two"), "blahonga\n").unwrap();
        run(ctx.dir.path(), &["git", "add", "file-one"]);
        let state = ctx.init_state();
        (ctx, state)
    }

    #[test]
    pub(crate) fn stash_menu() {
        let (mut ctx, mut state) = setup();
        state.update(&mut ctx.term, &keys("z")).unwrap();
        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    pub(crate) fn stash_prompt() {
        let (mut ctx, mut state) = setup();
        state.update(&mut ctx.term, &keys("zz")).unwrap();
        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    pub(crate) fn stash() {
        let (mut ctx, mut state) = setup();
        state.update(&mut ctx.term, &keys("zztest<enter>")).unwrap();
        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    pub(crate) fn stash_index_prompt() {
        let (mut ctx, mut state) = setup();
        state.update(&mut ctx.term, &keys("zi")).unwrap();
        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    pub(crate) fn stash_index() {
        let (mut ctx, mut state) = setup();
        state.update(&mut ctx.term, &keys("zitest<enter>")).unwrap();
        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    pub(crate) fn stash_working_tree_prompt() {
        let (mut ctx, mut state) = setup();
        state.update(&mut ctx.term, &keys("zw")).unwrap();
        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    pub(crate) fn stash_working_tree() {
        let (mut ctx, mut state) = setup();
        state.update(&mut ctx.term, &keys("zwtest<enter>")).unwrap();
        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    pub(crate) fn stash_working_tree_when_everything_is_staged() {
        let (mut ctx, mut state) = setup();
        state.update(&mut ctx.term, &keys("jszw")).unwrap();
        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    pub(crate) fn stash_working_tree_when_nothing_is_staged() {
        let mut ctx = TestContext::setup_clone(80, 20);
        fs::write(ctx.dir.child("file-one"), "blahonga\n").unwrap();
        let mut state = ctx.init_state();

        state.update(&mut ctx.term, &keys("zwtest<enter>")).unwrap();
        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    pub(crate) fn stash_keeping_index_prompt() {
        let (mut ctx, mut state) = setup();
        state.update(&mut ctx.term, &keys("zx")).unwrap();
        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    pub(crate) fn stash_keeping_index() {
        let (mut ctx, mut state) = setup();
        state.update(&mut ctx.term, &keys("zxtest<enter>")).unwrap();
        insta::assert_snapshot!(ctx.redact_buffer());
    }

    fn setup_two_stashes() -> (TestContext, State) {
        let (mut ctx, mut state) = setup();
        state
            .update(&mut ctx.term, &keys("zifile-one<enter>zzfile-two<enter>"))
            .unwrap();
        (ctx, state)
    }

    #[test]
    pub(crate) fn stash_pop_prompt() {
        let (mut ctx, mut state) = setup_two_stashes();
        state.update(&mut ctx.term, &keys("zp")).unwrap();
        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    pub(crate) fn stash_pop() {
        let (mut ctx, mut state) = setup_two_stashes();
        state.update(&mut ctx.term, &keys("zp1<enter>")).unwrap();
        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    pub(crate) fn stash_pop_default() {
        let (mut ctx, mut state) = setup_two_stashes();
        state.update(&mut ctx.term, &keys("zp<enter>")).unwrap();
        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    pub(crate) fn stash_apply_prompt() {
        let (mut ctx, mut state) = setup_two_stashes();
        state.update(&mut ctx.term, &keys("za")).unwrap();
        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    pub(crate) fn stash_apply() {
        let (mut ctx, mut state) = setup_two_stashes();
        state.update(&mut ctx.term, &keys("za1<enter>")).unwrap();
        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    pub(crate) fn stash_apply_default() {
        let (mut ctx, mut state) = setup_two_stashes();
        state.update(&mut ctx.term, &keys("za<enter>")).unwrap();
        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    pub(crate) fn stash_drop_prompt() {
        let (mut ctx, mut state) = setup_two_stashes();
        state.update(&mut ctx.term, &keys("zk")).unwrap();
        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    pub(crate) fn stash_drop() {
        let (mut ctx, mut state) = setup_two_stashes();
        state.update(&mut ctx.term, &keys("zk1<enter>")).unwrap();
        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    pub(crate) fn stash_drop_default() {
        let (mut ctx, mut state) = setup_two_stashes();
        state.update(&mut ctx.term, &keys("zk<enter>")).unwrap();
        insta::assert_snapshot!(ctx.redact_buffer());
    }
}

mod discard {
    use super::*;

    #[test]
    pub(crate) fn discard_branch_confirm_prompt() {
        let ctx = {
            let mut ctx = TestContext::setup_clone(80, 10);
            run(ctx.dir.path(), &["git", "branch", "asd"]);
            let mut state = ctx.init_state();

            state.update(&mut ctx.term, &keys("yjK")).unwrap();
            ctx
        };
        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    pub(crate) fn discard_branch_yes() {
        let mut ctx = TestContext::setup_clone(80, 10);
        run(ctx.dir.path(), &["git", "branch", "asd"]);
        let mut state = ctx.init_state();

        state.update(&mut ctx.term, &keys("yjKy")).unwrap();
        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    pub(crate) fn discard_branch_no() {
        let mut ctx = TestContext::setup_clone(80, 10);
        run(ctx.dir.path(), &["git", "branch", "asd"]);
        let mut state = ctx.init_state();

        state.update(&mut ctx.term, &keys("yjKn")).unwrap();
        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    pub(crate) fn discard_untracked_file() {
        let mut ctx = TestContext::setup_clone(80, 10);
        run(ctx.dir.path(), &["touch", "some-file"]);
        let mut state = ctx.init_state();

        state.update(&mut ctx.term, &keys("jjKy")).unwrap();
        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    pub(crate) fn discard_untracked_staged_file() {
        let mut ctx = TestContext::setup_clone(80, 15);
        run(ctx.dir.path(), &["touch", "some-file"]);
        run(ctx.dir.path(), &["git", "add", "some-file"]);
        let mut state = ctx.init_state();

        state.update(&mut ctx.term, &keys("jsjKy")).unwrap();
        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    pub(crate) fn discard_unstaged_delta() {
        let mut ctx = TestContext::setup_clone(80, 10);
        commit(ctx.dir.path(), "file-one", "FOO\nBAR\n");
        fs::write(ctx.dir.child("file-one"), "blahonga\n").unwrap();

        let mut state = ctx.init_state();

        state.update(&mut ctx.term, &keys("jjKy")).unwrap();
        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    pub(crate) fn discard_unstaged_hunk() {
        let mut ctx = TestContext::setup_clone(80, 10);
        commit(ctx.dir.path(), "file-one", "FOO\nBAR\n");
        fs::write(ctx.dir.child("file-one"), "blahonga\n").unwrap();

        let mut state = ctx.init_state();

        state.update(&mut ctx.term, &keys("jj<tab>jKy")).unwrap();
        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    pub(crate) fn discard_staged_file() {
        let mut ctx = TestContext::setup_clone(80, 10);
        commit(ctx.dir.path(), "file-one", "FOO\nBAR\n");
        fs::write(ctx.dir.child("file-one"), "blahonga\n").unwrap();
        run(ctx.dir.path(), &["git", "add", "."]);

        let mut state = ctx.init_state();

        state.update(&mut ctx.term, &keys("jjKy")).unwrap();
        insta::assert_snapshot!(ctx.redact_buffer());
    }

    // FIXME Deleting branches doesn't work with the test-setup
    // #[test]
    // fn discard_branch() {
    //     let mut ctx = TestContext::setup_clone(80, 10);
    //     let mut state = ctx.init_state();
    //     state
    //         .update(&mut ctx.term, &keys("yjKy"))
    //         .unwrap();
    //     insta::assert_snapshot!(ctx.redact_buffer());
    // }
}

mod reset {
    use super::*;

    #[test]
    pub(crate) fn reset_menu() {
        let mut ctx = TestContext::setup_clone(80, 10);
        commit(ctx.dir.path(), "unwanted-file", "");

        let mut state = ctx.init_state();
        state.update(&mut ctx.term, &keys("lljX")).unwrap();
        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    pub(crate) fn reset_soft_prompt() {
        let mut ctx = TestContext::setup_clone(80, 10);
        commit(ctx.dir.path(), "unwanted-file", "");

        let mut state = ctx.init_state();
        state.update(&mut ctx.term, &keys("lljXsq")).unwrap();
        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    pub(crate) fn reset_soft() {
        let mut ctx = TestContext::setup_clone(80, 10);
        commit(ctx.dir.path(), "unwanted-file", "");

        let mut state = ctx.init_state();
        state.update(&mut ctx.term, &keys("lljXs<enter>q")).unwrap();
        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    pub(crate) fn reset_mixed() {
        let mut ctx = TestContext::setup_clone(80, 10);
        commit(ctx.dir.path(), "unwanted-file", "");

        let mut state = ctx.init_state();
        state.update(&mut ctx.term, &keys("lljXm<enter>q")).unwrap();
        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    fn reset_hard() {
        let mut ctx = TestContext::setup_clone(80, 10);
        commit(ctx.dir.path(), "unwanted-file", "");

        let mut state = ctx.init_state();
        state.update(&mut ctx.term, &keys("lljXh<enter>q")).unwrap();
        insta::assert_snapshot!(ctx.redact_buffer());
    }
}

#[test]
fn show_refs() {
    let mut ctx = TestContext::setup_clone(80, 10);
    run(ctx.dir.path(), &["git", "tag", "same-name"]);
    run(ctx.dir.path(), &["git", "checkout", "-b", "same-name"]);

    let mut state = ctx.init_state();
    state.update(&mut ctx.term, &keys("y")).unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
}

mod checkout {
    use super::*;

    #[test]
    pub(crate) fn checkout_menu() {
        let mut ctx = TestContext::setup_clone(80, 10);
        run(ctx.dir.path(), &["git", "branch", "other-branch"]);

        let mut state = ctx.init_state();
        state.update(&mut ctx.term, &keys("yjb")).unwrap();
        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    pub(crate) fn switch_branch_selected() {
        let mut ctx = TestContext::setup_clone(80, 10);
        run(ctx.dir.path(), &["git", "branch", "other-branch"]);

        let mut state = ctx.init_state();
        state.update(&mut ctx.term, &keys("yjjbb<enter>")).unwrap();
        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    pub(crate) fn switch_branch_input() {
        let mut ctx = TestContext::setup_clone(80, 10);
        run(ctx.dir.path(), &["git", "branch", "hi"]);

        let mut state = ctx.init_state();
        state
            .update(&mut ctx.term, &keys("yjjbbhi<enter>"))
            .unwrap();
        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    pub(crate) fn checkout_new_branch() {
        let mut ctx = TestContext::setup_clone(80, 10);

        let mut state = ctx.init_state();
        state
            .update(&mut ctx.term, &keys("bcf<esc>bcx<enter>"))
            .unwrap();
        insta::assert_snapshot!(ctx.redact_buffer());
    }
}

#[test]
fn updated_externally() {
    let mut ctx = TestContext::setup_init(80, 20);
    fs::write(ctx.dir.child("b"), "test").unwrap();

    let mut state = ctx.init_state();
    state.update(&mut ctx.term, &keys("jjsj")).unwrap();

    fs::write(ctx.dir.child("a"), "test").unwrap();

    state.update(&mut ctx.term, &keys("g")).unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn stage_last_hunk_of_first_delta() {
    let mut ctx = TestContext::setup_clone(80, 20);
    commit(ctx.dir.path(), "file-one", "asdf\nblahonga\n");
    commit(ctx.dir.path(), "file-two", "FOO\nBAR\n");
    fs::write(ctx.dir.child("file-one"), "blahonga\n").unwrap();
    fs::write(ctx.dir.child("file-two"), "blahonga\n").unwrap();

    let mut state = ctx.init_state();
    state.update(&mut ctx.term, &keys("jj<tab>js")).unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn go_down_past_collapsed() {
    let mut ctx = TestContext::setup_init(80, 20);
    commit(ctx.dir.path(), "file-one", "asdf\nblahonga\n");
    commit(ctx.dir.path(), "file-two", "FOO\nBAR\n");
    fs::write(ctx.dir.child("file-one"), "blahonga\n").unwrap();
    fs::write(ctx.dir.child("file-two"), "blahonga\n").unwrap();

    let mut state = ctx.init_state();
    state.update(&mut ctx.term, &keys("jjj")).unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
}

mod scroll {
    use super::*;

    fn setup_scroll() -> (TestContext, crate::state::State) {
        let mut ctx = TestContext::setup_init(80, 20);
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
    let mut ctx = TestContext::setup_clone(80, 20);
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
fn quit() {
    let mut ctx = TestContext::setup_init(80, 20);

    // TODO init_state should probably accept `Config` as an arg?
    let mut state = ctx.init_state();
    state.update(&mut ctx.term, &keys("qy")).unwrap();
    assert!(state.quit);
}
