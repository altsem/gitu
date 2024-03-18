use crate::helpers::{clone_and_commit, commit, ctrl, key, key_code, run, TestContext};
use crossterm::event::KeyCode;
use itertools::Itertools;
use std::fs;

mod helpers;

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
    state.update(&mut ctx.term, &[key('h')]).unwrap();
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
    state
        .update(&mut ctx.term, &[key('j'), key('j'), key_code(KeyCode::Tab)])
        .unwrap();

    insta::assert_snapshot!(ctx.redact_buffer());
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
    fn stage_all_untracked() {
        let mut ctx = TestContext::setup_init(80, 20);
        run(ctx.dir.path(), &["touch", "file-a"]);
        run(ctx.dir.path(), &["touch", "file-b"]);

        let mut state = ctx.init_state();
        state.update(&mut ctx.term, &[key('j'), key('s')]).unwrap();

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
    state.update(&mut ctx.term, &[key('l'), key('l')]).unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn log_other() {
    let mut ctx = TestContext::setup_clone(80, 20);
    commit(ctx.dir.path(), "this-should-be-at-the-top", "");
    commit(ctx.dir.path(), "this-should-not-be-visible", "");

    let mut state = ctx.init_state();
    state
        .update(
            &mut ctx.term,
            &[key('l'), key('l'), key('j'), key('l'), key('o')],
        )
        .unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn show() {
    let mut ctx = TestContext::setup_clone(80, 20);
    commit(ctx.dir.path(), "firstfile", "This should be visible\n");

    let mut state = ctx.init_state();
    state
        .update(
            &mut ctx.term,
            &[key('l'), key('l'), key_code(KeyCode::Enter)],
        )
        .unwrap();
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
    state.update(&mut ctx.term, &[key('g')]).unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn new_commit() {
    let mut ctx = TestContext::setup_clone(80, 10);
    commit(ctx.dir.path(), "new-file", "");

    ctx.init_state();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn push() {
    let mut ctx = TestContext::setup_clone(80, 10);
    commit(ctx.dir.path(), "new-file", "");

    let mut state = ctx.init_state();
    state.update(&mut ctx.term, &[key('P'), key('p')]).unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn fetch_all() {
    let mut ctx = TestContext::setup_clone(80, 10);
    clone_and_commit(&ctx.remote_dir, "remote-file", "hello");

    let mut state = ctx.init_state();
    state.update(&mut ctx.term, &[key('f'), key('a')]).unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn pull() {
    let mut ctx = TestContext::setup_clone(80, 10);
    clone_and_commit(&ctx.remote_dir, "remote-file", "hello");

    let mut state = ctx.init_state();
    state.update(&mut ctx.term, &[key('F'), key('p')]).unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
}

mod discard {
    use crate::helpers::commit;
    use crate::helpers::key;
    use crate::helpers::key_code;
    use crate::helpers::run;
    use crate::helpers::TestContext;
    use crossterm::event::KeyCode;
    use std::fs;

    #[test]
    pub(crate) fn discard_branch_confirm_prompt() {
        let ctx = {
            let mut ctx = TestContext::setup_clone(80, 10);
            run(ctx.dir.path(), &["git", "branch", "asd"]);
            let mut state = ctx.init_state();

            state
                .update(&mut ctx.term, &[key('y'), key('j'), key('K')])
                .unwrap();
            ctx
        };
        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    pub(crate) fn discard_branch_yes() {
        let mut ctx = TestContext::setup_clone(80, 10);
        run(ctx.dir.path(), &["git", "branch", "asd"]);
        let mut state = ctx.init_state();

        state
            .update(&mut ctx.term, &[key('y'), key('j'), key('K'), key('y')])
            .unwrap();
        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    pub(crate) fn discard_branch_no() {
        let mut ctx = TestContext::setup_clone(80, 10);
        run(ctx.dir.path(), &["git", "branch", "asd"]);
        let mut state = ctx.init_state();

        state
            .update(&mut ctx.term, &[key('y'), key('j'), key('K'), key('n')])
            .unwrap();
        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    pub(crate) fn discard_untracked_file() {
        let mut ctx = TestContext::setup_clone(80, 10);
        run(ctx.dir.path(), &["touch", "some-file"]);
        let mut state = ctx.init_state();

        state
            .update(&mut ctx.term, &[key('j'), key('j'), key('K'), key('y')])
            .unwrap();
        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    pub(crate) fn discard_unstaged_delta() {
        let mut ctx = TestContext::setup_clone(80, 10);
        commit(ctx.dir.path(), "file-one", "FOO\nBAR\n");
        fs::write(ctx.dir.child("file-one"), "blahonga\n").unwrap();

        let mut state = ctx.init_state();

        state
            .update(&mut ctx.term, &[key('j'), key('j'), key('K'), key('y')])
            .unwrap();
        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    pub(crate) fn discard_unstaged_hunk() {
        let mut ctx = TestContext::setup_clone(80, 10);
        commit(ctx.dir.path(), "file-one", "FOO\nBAR\n");
        fs::write(ctx.dir.child("file-one"), "blahonga\n").unwrap();

        let mut state = ctx.init_state();

        state
            .update(
                &mut ctx.term,
                &[
                    key('j'),
                    key('j'),
                    key_code(KeyCode::Tab),
                    key('j'),
                    key('K'),
                    key('y'),
                ],
            )
            .unwrap();
        insta::assert_snapshot!(ctx.redact_buffer());
    }

    // FIXME Deleting branches doesn't work with the test-setup
    // #[test]
    // fn discard_branch() {
    //     let mut ctx = TestContext::setup_clone(80, 10);
    //     let mut state = ctx.init_state();
    //     state
    //         .update(&mut ctx.term, &[key('y'), key('j'), key('K'), key('y')])
    //         .unwrap();
    //     insta::assert_snapshot!(ctx.redact_buffer());
    // }
}

mod reset {
    use crate::helpers::commit;
    use crate::helpers::key;
    use crate::helpers::TestContext;

    #[test]
    pub(crate) fn reset_menu() {
        let mut ctx = TestContext::setup_clone(80, 10);
        commit(ctx.dir.path(), "unwanted-file", "");

        let mut state = ctx.init_state();
        state
            .update(&mut ctx.term, &[key('l'), key('l'), key('j'), key('X')])
            .unwrap();
        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    pub(crate) fn reset_soft() {
        let mut ctx = TestContext::setup_clone(80, 10);
        commit(ctx.dir.path(), "unwanted-file", "");

        let mut state = ctx.init_state();
        state
            .update(
                &mut ctx.term,
                &[key('l'), key('l'), key('j'), key('X'), key('s'), key('q')],
            )
            .unwrap();
        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    pub(crate) fn reset_mixed() {
        let mut ctx = TestContext::setup_clone(80, 10);
        commit(ctx.dir.path(), "unwanted-file", "");

        let mut state = ctx.init_state();
        state
            .update(
                &mut ctx.term,
                &[key('l'), key('l'), key('j'), key('X'), key('m'), key('q')],
            )
            .unwrap();
        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    fn reset_hard() {
        let mut ctx = TestContext::setup_clone(80, 10);
        commit(ctx.dir.path(), "unwanted-file", "");

        let mut state = ctx.init_state();
        state
            .update(
                &mut ctx.term,
                &[key('l'), key('l'), key('j'), key('X'), key('h'), key('q')],
            )
            .unwrap();
        insta::assert_snapshot!(ctx.redact_buffer());
    }
}

#[test]
fn show_refs() {
    let mut ctx = TestContext::setup_clone(80, 10);
    run(ctx.dir.path(), &["git", "tag", "same-name"]);
    run(ctx.dir.path(), &["git", "checkout", "-b", "same-name"]);

    let mut state = ctx.init_state();
    state.update(&mut ctx.term, &[key('y')]).unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
}

mod checkout {
    use crate::helpers::key;
    use crate::helpers::key_code;
    use crate::helpers::run;
    use crate::helpers::TestContext;
    use crossterm::event::KeyCode;

    #[test]
    pub(crate) fn checkout_menu() {
        let mut ctx = TestContext::setup_clone(80, 10);
        run(ctx.dir.path(), &["git", "branch", "other-branch"]);

        let mut state = ctx.init_state();
        state
            .update(&mut ctx.term, &[key('y'), key('j'), key('b')])
            .unwrap();
        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    pub(crate) fn switch_branch_selected() {
        let mut ctx = TestContext::setup_clone(80, 10);
        run(ctx.dir.path(), &["git", "branch", "other-branch"]);

        let mut state = ctx.init_state();
        state
            .update(
                &mut ctx.term,
                &[
                    key('y'),
                    key('j'),
                    key('j'),
                    key('b'),
                    key('b'),
                    key_code(KeyCode::Enter),
                ],
            )
            .unwrap();
        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    pub(crate) fn switch_branch_input() {
        let mut ctx = TestContext::setup_clone(80, 10);
        run(ctx.dir.path(), &["git", "branch", "hi"]);

        let mut state = ctx.init_state();
        state
            .update(
                &mut ctx.term,
                &[
                    key('y'),
                    key('j'),
                    key('j'),
                    key('b'),
                    key('b'),
                    key('h'),
                    key('i'),
                    key_code(KeyCode::Enter),
                ],
            )
            .unwrap();
        insta::assert_snapshot!(ctx.redact_buffer());
    }

    #[test]
    pub(crate) fn checkout_new_branch() {
        let mut ctx = TestContext::setup_clone(80, 10);

        let mut state = ctx.init_state();
        state
            .update(
                &mut ctx.term,
                &[
                    key('b'),
                    key('c'),
                    key('f'),
                    // Don't want to create branch 'f', try again
                    key_code(KeyCode::Esc),
                    key('b'),
                    key('c'),
                    key('x'),
                    key_code(KeyCode::Enter),
                ],
            )
            .unwrap();
        insta::assert_snapshot!(ctx.redact_buffer());
    }
}

#[test]
fn updated_externally() {
    let mut ctx = TestContext::setup_init(80, 20);
    fs::write(ctx.dir.child("b"), "test").unwrap();

    let mut state = ctx.init_state();
    state
        .update(&mut ctx.term, &[key('j'), key('j'), key('s'), key('j')])
        .unwrap();

    fs::write(ctx.dir.child("a"), "test").unwrap();

    state.update(&mut ctx.term, &[key('g')]).unwrap();
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
    state
        .update(
            &mut ctx.term,
            &[
                key('j'),
                key('j'),
                key_code(KeyCode::Tab),
                key('j'),
                key('s'),
            ],
        )
        .unwrap();
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
    state
        .update(&mut ctx.term, &[key('j'), key('j'), key('j')])
        .unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn scroll_down() {
    let mut ctx = TestContext::setup_init(80, 20);
    commit(ctx.dir.path(), "file-one", "");
    fs::write(
        ctx.dir.child("file-one"),
        (1..=100).map(|i| format!("line {}", i)).join("\n"),
    )
    .unwrap();

    let mut state = ctx.init_state();
    state
        .update(
            &mut ctx.term,
            &[key('j'), key('j'), key_code(KeyCode::Tab), ctrl('d')],
        )
        .unwrap();
    insta::assert_snapshot!(ctx.redact_buffer());
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
