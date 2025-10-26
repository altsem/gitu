use super::*;

#[test]
fn push_menu_no_remote_or_upstream_set() {
    let ctx = setup_clone!();
    run(&ctx.dir, &["git", "branch", "--unset-upstream"]);
    snapshot!(ctx, "P");
}

#[test]
fn push_menu_existing_push_remote_and_upstream() {
    let ctx = setup_clone!();
    run(
        &ctx.dir,
        &["git", "config", "branch.main.pushRemote", "origin"],
    );
    snapshot!(ctx, "P");
}

#[test]
fn push_upstream() {
    let ctx = setup_clone!();
    commit(&ctx.dir, "new-file", "");
    snapshot!(ctx, "Pu");
}

#[test]
fn push_push_remote() {
    let ctx = setup_clone!();
    run(
        &ctx.dir,
        &["git", "config", "branch.main.pushRemote", "origin"],
    );

    snapshot!(ctx, "Pp");
}

#[test]
fn push_upstream_prompt() {
    let ctx = setup_clone!();
    run(&ctx.dir, &["git", "branch", "--unset-upstream"]);
    snapshot!(ctx, "Pu");
}

#[test]
fn push_push_remote_prompt() {
    let ctx = setup_clone!();
    commit(&ctx.dir, "new-file", "");
    snapshot!(ctx, "Pp");
}

#[test]
fn push_setup_upstream() {
    let ctx = setup_clone!();
    run(&ctx.dir, &["git", "checkout", "-b", "new-branch"]);
    commit(&ctx.dir, "new-file", "");
    snapshot!(ctx, "Pumain<enter>P");
}

#[test]
fn push_setup_upstream_same_as_head() {
    let ctx = setup_clone!();
    run(&ctx.dir, &["git", "checkout", "-b", "new-branch"]);
    commit(&ctx.dir, "new-file", "");
    snapshot!(ctx, "Punew-branch<enter>");
}

#[test]
fn push_setup_push_remote() {
    let ctx = setup_clone!();
    snapshot!(ctx, "Pporigin<enter>P");
}

#[test]
fn force_push() {
    let ctx = setup_clone!();
    commit(&ctx.dir, "new-file", "");
    snapshot!(ctx, "P-fu");
}

#[test]
fn open_push_menu_after_dash_input() {
    let ctx = setup_clone!();
    commit(&ctx.dir, "new-file", "");
    snapshot!(ctx, "-P");
}

#[test]
fn push_menu_no_branch() {
    snapshot!(setup_clone!(), "bbb66a0bf<enter>Pp");
}

#[test]
fn push_elsewhere_prompt() {
    snapshot!(setup_clone!(), "Pe");
}

#[test]
fn push_elsewhere() {
    snapshot!(setup_clone!(), "Peorigin<enter>");
}
