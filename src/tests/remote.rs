use git2::{Buf, Error, Repository};

use crate::git::remote::*;
use crate::tests::helpers::{run, RepoTestContext};

fn get_head_name(repo: &Repository) -> String {
    repo.head().unwrap().name().unwrap().into()
}

fn get_branch_remote(repo: &Repository) -> Result<Buf, Error> {
    repo.branch_upstream_remote(&get_head_name(repo))
}

fn get_branch_remote_str(repo: &Repository) -> String {
    get_branch_remote(repo).unwrap().as_str().unwrap().into()
}

#[test]
fn set_push_remote_basic() {
    let ctx = RepoTestContext::setup_clone();

    let repo = ctx.local_repo;

    let push_remote = get_push_remote(&repo).unwrap();
    assert_eq!(push_remote, None);

    let remote_name = get_branch_remote_str(&repo);
    let remote = repo.find_remote(&remote_name).unwrap();
    set_push_remote(&repo, Some(&remote)).unwrap();
    let push_remote = get_push_remote(&repo).unwrap();
    assert_eq!(push_remote, Some(remote_name));

    set_push_remote(&repo, None).unwrap();
    let push_remote = get_push_remote(&repo).unwrap();
    assert_eq!(push_remote, None);
}

#[test]
fn get_upstream_basic() {
    let ctx = RepoTestContext::setup_clone();
    let repo = ctx.local_repo;
    let upstream = get_upstream(&repo).unwrap().unwrap();
    assert_eq!(upstream.name().unwrap().unwrap(), "origin/main");

    run(ctx.dir.path(), &["git", "branch", "--unset-upstream"]);
    let upstream = get_upstream(&repo).unwrap();
    assert!(upstream.is_none());
}

#[test]
fn get_upstream_components_of_remote_branch() {
    let ctx = RepoTestContext::setup_clone();
    let repo = ctx.local_repo;

    let (remote, branch) = get_upstream_components(&repo).unwrap().unwrap();
    assert_eq!(remote, "origin");
    assert_eq!(branch, "main");
}

#[test]
fn get_upstream_components_of_feature_branch() {
    let ctx = RepoTestContext::setup_clone();
    let repo = ctx.local_repo;
    run(ctx.dir.path(), &["git", "checkout", "-b", "feature-branch"]);
    run(
        ctx.dir.path(),
        &["git", "branch", "--set-upstream-to", "main"],
    );

    let (remote, branch) = get_upstream_components(&repo).unwrap().unwrap();
    assert_eq!(remote, ".");
    assert_eq!(branch, "main");
}
