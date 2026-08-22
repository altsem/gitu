use super::*;

fn setup(ctx: TestContext) -> TestContext {
    commit(&ctx.dir, "firstfile", "");
    commit(&ctx.dir, "secondfile", "");
    commit(&ctx.dir, "thirdfile", "");
    ctx
}

#[test]
fn search_prompt() {
    snapshot!(setup(setup_clone!()), "/");
}

#[test]
fn search_forward() {
    snapshot!(setup(setup_clone!()), "/second<enter>");
}

#[test]
fn search_next_and_previous() {
    snapshot!(setup(setup_clone!()), "/file<enter>nnN");
}

#[test]
fn search_aborted() {
    snapshot!(setup(setup_clone!()), "/second<esc>");
}

#[test]
fn search_without_match() {
    snapshot!(setup(setup_clone!()), "/nonesuch<enter>");
}

#[test]
fn search_repeat_without_a_previous_search() {
    snapshot!(setup(setup_clone!()), "n");
}

fn marked(ctx: TestContext, input: &str) -> Vec<String> {
    let mut ctx = setup(ctx);
    let mut app = ctx.init_app();
    ctx.update(&mut app, keys(input));
    ctx.highlighted()
}

#[test]
fn every_match_on_screen_is_marked() {
    assert_eq!(
        vec!["file", "file", "file", "file"],
        marked(setup_clone!(), "/file<enter>")
    );
}

/// The cursor is left on one match, but every one of them is marked.
#[test]
fn repeating_a_search_leaves_the_marks_alone() {
    assert_eq!(
        vec!["file", "file", "file", "file"],
        marked(setup_clone!(), "/file<enter>nn")
    );
}

/// A query written in lowercase says nothing about case, so it matches either.
#[test]
fn a_lowercase_query_matches_whatever_the_case() {
    assert_eq!(
        vec!["Author", "Author", "Author", "Author"],
        marked(setup_clone!(), "/author<enter>")
    );
}

/// Uppercase in the query is taken to be meant, as vim's `smartcase` does.
#[test]
fn an_uppercase_query_matches_that_case_only() {
    assert!(marked(setup_clone!(), "/Add<enter>").is_empty());
    assert_eq!(
        vec!["add", "add", "add", "add"],
        marked(setup_clone!(), "/add<enter>")
    );
}

/// The branch and the summary are separate spans of a commit row, and both
/// the checked out branch and the remote's row read as `main add`.
#[test]
fn a_match_running_from_one_span_into_the_next_is_marked_whole() {
    assert_eq!(
        vec!["main add", "main add"],
        marked(setup_clone!(), "/main add<enter>")
    );
}

/// The author sits outside the group holding the summary, so the two are never
/// one match, however adjacent they end up looking.
#[test]
fn text_either_side_of_a_layout_break_is_not_one_match() {
    assert!(marked(setup_clone!(), "/thirdfileAuthor<enter>").is_empty());
}

/// Nothing matched, and the query the command log echoes back is no match of
/// its own.
#[test]
fn nothing_is_marked_without_a_match() {
    assert!(marked(setup_clone!(), "/nonesuch<enter>").is_empty());
}

#[test]
fn aborting_the_prompt_clears_the_marks() {
    assert!(marked(setup_clone!(), "/file<enter>/<esc>").is_empty());
}

/// An empty query has nothing to search for, so it clears instead.
#[test]
fn an_empty_query_clears_the_marks() {
    assert!(marked(setup_clone!(), "/file<enter>/<enter>").is_empty());
}

/// Cleared means forgotten, so there is nothing left for `n` to repeat.
#[test]
fn a_cleared_search_is_not_repeatable() {
    snapshot!(setup(setup_clone!()), "/file<enter>/<esc>n");
}

/// And the same, whichever way it was cleared.
#[test]
fn a_search_cleared_by_an_empty_query_is_not_repeatable() {
    snapshot!(setup(setup_clone!()), "/file<enter>/<enter>n");
}

/// The commit menu draws the selected item as a row of its own, which is not
/// what was searched and so is left unmarked.
#[test]
fn only_the_items_on_screen_are_marked() {
    assert_eq!(
        vec!["file", "file", "file", "file"],
        marked(setup_clone!(), "/file<enter>c")
    );
}

/// A hunk line is searchable, even when collapsed.
#[test]
fn search_hunk_line() {
    let ctx = setup_clone!();
    commit(&ctx.dir, "testfile", "one\ntwo\nthree\n");
    std::fs::write(ctx.dir.join("testfile"), "one\ntwo\nfour\n").unwrap();
    snapshot!(ctx, "/four<enter>");
}

/// A diff's context lines are unselectable, and are most of what a diff has
/// written on it. Search reaches them, so that what is marked can be moved to.
#[test]
fn search_context_line() {
    let ctx = setup_clone!();
    commit(&ctx.dir, "testfile", "one\ntwo\nthree\n");
    std::fs::write(ctx.dir.join("testfile"), "one\ntwo\nfour\n").unwrap();
    snapshot!(ctx, "jj<tab>/two<enter>");
}

#[test]
fn a_context_line_is_marked() {
    let ctx = setup_clone!();
    commit(&ctx.dir, "testfile", "one\ntwo\nthree\n");
    std::fs::write(ctx.dir.join("testfile"), "one\ntwo\nfour\n").unwrap();

    let mut ctx = ctx;
    let mut app = ctx.init_app();
    ctx.update(&mut app, keys("jj<tab>/two<enter>"));

    assert_eq!(vec!["two"], ctx.highlighted());
}
