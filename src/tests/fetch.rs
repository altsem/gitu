use super::*;

#[test]
fn fetch_menu_no_remote_or_upstream_set() {
    let ctx = setup_clone!();
    run(&ctx.dir, &["git", "branch", "--unset-upstream"]);
    snapshot!(ctx, "f");
}

#[test]
fn fetch_menu_existing_push_remote_and_upstream() {
    let ctx = setup_clone!();
    run(
        &ctx.dir,
        &["git", "config", "branch.main.pushRemote", "origin"],
    );
    snapshot!(ctx, "f");
}

#[test]
fn fetch_from_elsewhere_prompt() {
    snapshot!(setup_clone!(), "fe");
}

#[test]
fn fetch_from_elsewhere() {
    snapshot!(setup_clone!(), "feorigin<enter>");
}

#[test]
fn fetch_from_upstream_prompt() {
    let ctx = setup_clone!();
    run(&ctx.dir, &["git", "branch", "--unset-upstream"]);
    snapshot!(ctx, "fu");
}

#[test]
fn fetch_from_upstream() {
    snapshot!(setup_clone!(), "fu");
}

#[test]
fn fetch_from_push_remote_prompt() {
    snapshot!(setup_clone!(), "fp");
}

#[test]
fn fetch_from_push_remote() {
    let ctx = setup_clone!();
    run(
        &ctx.dir,
        &["git", "config", "branch.main.pushRemote", "origin"],
    );
    snapshot!(ctx, "fp");
}
