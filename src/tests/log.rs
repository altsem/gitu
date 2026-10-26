use super::*;

fn setup(ctx: TestContext) -> TestContext {
    commit(&ctx.dir, "third commit", "");
    commit(&ctx.dir, "second commit", "");
    commit(&ctx.dir, "first commit", "");
    ctx
}

#[test]
fn limit_prompt() {
    snapshot!(setup(setup_clone!()), "l-n-n");
}

#[test]
fn limit_set_10() {
    snapshot!(setup(setup_clone!()), "l-n-n10<enter>");
}

#[test]
fn limit_invalid() {
    snapshot!(setup(setup_clone!()), "l-n-nfff<enter>");
}

#[test]
fn limit_2_commits() {
    snapshot!(setup(setup_clone!()), "l-n-n2<enter>l");
}

#[test]
fn limit_2_commits_other() {
    snapshot!(setup(setup_clone!()), "l-n-n2<enter>l");
}

#[test]
fn grep_prompt() {
    snapshot!(setup(setup_clone!()), "l-F");
}

#[test]
fn grep_set_example() {
    snapshot!(setup(setup_clone!()), "l-Fexample<enter>");
}

#[test]
fn grep_second() {
    snapshot!(setup(setup_clone!()), "l-Fsecond<enter>l");
}

#[test]
fn grep_no_match() {
    snapshot!(setup(setup_clone!()), "l-Fdoesntexist<enter>l");
}

#[test]
fn grep_second_other() {
    snapshot!(setup(setup_clone!()), "l-Fsecond<enter>omain<enter>");
}

#[test]
fn log_other_prompt() {
    snapshot!(setup(setup_clone!()), "lljlo");
}

#[test]
fn log_other() {
    snapshot!(setup(setup_clone!()), "lljlo<enter>");
}

#[test]
fn log_other_input() {
    snapshot!(setup(setup_clone!()), "lomain~1<enter>");
}

#[test]
fn log_other_invalid() {
    snapshot!(setup(setup_clone!()), "lo <enter>");
}

/// `git` with a fixed committer/author date, set per-process rather than
/// through the process-wide env that the shared helpers use, so parallel
/// tests can't clobber each other's dates.
fn git_at(ctx: &TestContext, date: &str, args: &[&str]) {
    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(&ctx.dir)
        .env("GIT_AUTHOR_DATE", date)
        .env("GIT_COMMITTER_DATE", date)
        .output()
        .unwrap_or_else(|e| panic!("failed to execute git {args:?}: {e}"));
    assert!(
        output.status.success(),
        "failed to execute git {args:?}. Output: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Like the shared `commit()` helper, but with a fixed commit date (see
/// [`git_at`]).
fn commit_at(ctx: &TestContext, date: &str, file_name: &str, contents: &str, message: &str) {
    fs::write(ctx.dir.join(file_name), contents).expect("error writing file");
    run(&ctx.dir, &["git", "add", file_name]);
    git_at(ctx, date, &["commit", "-m", message]);
}

#[test]
fn log_local_branches() {
    let ctx = setup_clone!();
    //          M (main)
    //         / \
    //      C4    C3 (sub)
    //      |     |
    //      C1 -  C2 (feature)
    //      |
    //     C0 (origin/main)
    //
    // A branch (feature) off main, another branch (sub) off that, and sub
    // merged back into main. Each commit gets a distinct committer date,
    // so the tree walk is a plain date-ordered one, like `git log`.
    commit_at(
        &ctx,
        "2024-03-01T10:00:00+00:00",
        "one",
        "1\n",
        "first commit",
    );
    run(&ctx.dir, &["git", "checkout", "-q", "-b", "feature"]);
    commit_at(
        &ctx,
        "2024-03-02T10:00:00+00:00",
        "two",
        "2\n",
        "start feature",
    );
    run(&ctx.dir, &["git", "checkout", "-q", "-b", "sub"]);
    commit_at(
        &ctx,
        "2024-03-03T10:00:00+00:00",
        "three",
        "3\n",
        "add to feature",
    );
    run(&ctx.dir, &["git", "checkout", "-q", "main"]);
    commit_at(
        &ctx,
        "2024-03-04T10:00:00+00:00",
        "four",
        "4\n",
        "more main work",
    );
    git_at(
        &ctx,
        "2024-03-05T10:00:00+00:00",
        &["merge", "-q", "--no-ff", "sub"],
    );

    snapshot!(ctx, "lL");
}

//          C3 (side)
//          |
//          C2 (main)      <- v1, and a stash (S) in log_all_refs
//          |
//          C1
//          |
//        C0 (origin/main)
//
// The local branches diverge at C1 (side has C3, main has C2), and
// origin/main points at the initial commit.
fn diverging_branches(ctx: &TestContext) {
    commit_at(
        ctx,
        "2024-03-01T10:00:00+00:00",
        "one",
        "1\n",
        "first commit",
    );
    run(&ctx.dir, &["git", "checkout", "-q", "-b", "side"]);
    commit_at(
        ctx,
        "2024-03-03T10:00:00+00:00",
        "three",
        "3\n",
        "third commit",
    );
    run(&ctx.dir, &["git", "checkout", "-q", "main"]);
    commit_at(
        ctx,
        "2024-03-02T10:00:00+00:00",
        "two",
        "2\n",
        "second commit",
    );
}

#[test]
fn log_local_branches_single_branch() {
    let ctx = setup_clone!();
    //          C4 (main)
    //        /      \
    //     C2        C3 (side)
    //      |         |
    //      C1 -------  (side merged back with --no-ff)
    //      |
    //     C0 (origin/main)
    //
    // With only one local branch, the tree view has a single root; it must
    // still show the graph (a plain log would hide the merged branch).
    commit_at(
        &ctx,
        "2024-03-01T10:00:00+00:00",
        "one",
        "1\n",
        "first commit",
    );
    run(&ctx.dir, &["git", "checkout", "-q", "-b", "side"]);
    commit_at(
        &ctx,
        "2024-03-02T10:00:00+00:00",
        "two",
        "2\n",
        "second commit",
    );
    run(&ctx.dir, &["git", "checkout", "-q", "main"]);
    commit_at(
        &ctx,
        "2024-03-03T10:00:00+00:00",
        "three",
        "3\n",
        "third commit",
    );
    git_at(
        &ctx,
        "2024-03-04T10:00:00+00:00",
        &["merge", "-q", "--no-ff", "side"],
    );
    run(&ctx.dir, &["git", "branch", "-d", "side"]);

    snapshot!(ctx, "lL");
}

#[test]
fn log_all_branches() {
    let ctx = setup_clone!();
    diverging_branches(&ctx);

    // Roots are every local and remote branch tip.
    snapshot!(ctx, "lb");
}

#[test]
fn log_all_refs() {
    let ctx = setup_clone!();
    diverging_branches(&ctx);
    // A tag on the main tip, and a stash on top of side (a ref that is
    // neither a branch, tag, nor remote branch).
    run(&ctx.dir, &["git", "tag", "v1"]);
    run(&ctx.dir, &["git", "checkout", "-q", "side"]);
    fs::write(ctx.dir.join("initial-file"), "hello\nmodified\n").expect("error writing file");
    git_at(&ctx, "2024-03-04T10:00:00+00:00", &["stash", "-q"]);

    // Roots are every reference.
    snapshot!(ctx, "la");
}

#[test]
fn log_empty_branch() {
    // Regression for #262: showing the log of a branch with no commits used to
    // panic ("index out of bounds") because the log screen had no items but the
    // cursor still indexed into it.
    let mut ctx = setup_clone!();
    run(&ctx.dir, &["rm", "-rf", ".git"]);
    run(&ctx.dir, &["rm", "initial-file"]);
    run(&ctx.dir, &["git", "init", "--initial-branch=main"]);

    let mut app = ctx.init_app();
    ctx.update(&mut app, keys("ll"));
    insta::assert_snapshot!(ctx.redact_buffer());
}
