use std::str::FromStr;
use git2::{Buf, Error, ErrorClass, Repository};

use crate::git::remote::set_upstream;
use crate::tests::helpers::{commit, RepoTestContext};

fn get_head_name(repo: &Repository) -> String {
    repo.head().unwrap().name().unwrap().into()
}

fn get_branch_merge(repo: &Repository) -> Result<Buf, Error> {
    repo.branch_upstream_name(&get_head_name(repo))
}

fn get_branch_merge_str(repo: &Repository) -> String {
    get_branch_merge(repo).unwrap().as_str().unwrap().into()
}

fn get_branch_remote(repo: &Repository) -> Result<Buf, Error> {
    repo.branch_upstream_remote(&get_head_name(repo))
}

fn get_branch_remote_str(repo: &Repository) -> String {
    get_branch_remote(repo).unwrap().as_str().unwrap().into()
}

fn branch_from_head(repo: &Repository, name: impl AsRef<str>) -> git2::Branch {
    repo
        .branch(
            name.as_ref(),
            &repo.head().unwrap().peel_to_commit().unwrap(),
            true,
        )
        .unwrap()
}

#[test]
fn remove_upstream() {
    let ctx = RepoTestContext::setup_init();

    let repo = ctx.local_repo;
    commit(repo.workdir().unwrap(), "file.txt", "content");

    set_upstream(&repo, None).unwrap();

    let e = get_branch_merge(&repo).map(|v| String::from_str(v.as_str().unwrap())).unwrap_err();
    assert!(e.class() == ErrorClass::Config, "Actual: {}", e);

    let e = get_branch_remote(&repo).map(|v| String::from_str(v.as_str().unwrap())).unwrap_err();

    assert!(e.class() == ErrorClass::Config, "Actual: {}", e);
}

#[test]
fn set_upstream_basic() {
    let ctx = RepoTestContext::setup_clone();

    let repo = ctx.local_repo;
    let remote_name = get_branch_remote_str(&repo);

    let upstream_branch = branch_from_head(&ctx.remote_repo, "upstream-branch");
    let upstream_ref = upstream_branch.into_reference();

    repo.find_remote(&remote_name)
        .unwrap()
        .fetch(&[upstream_ref.shorthand().unwrap()], None, None)
        .unwrap();

    let upstream_name = upstream_ref.shorthand().unwrap();
    let upstream_ref = repo
        .find_branch(
            &format!("{remote_name}/{upstream_name}"),
            git2::BranchType::Remote,
        )
        .unwrap()
        .into_reference();

    set_upstream(&repo, Some(&upstream_ref)).unwrap();

    let actual_merge = get_branch_merge_str(&repo);
    let actual_remote = get_branch_remote_str(&repo);

    assert_eq!(actual_merge, upstream_ref.name().unwrap());
    assert_eq!(actual_remote, remote_name);
}

#[test]
fn set_upstream_local() {
    let ctx = RepoTestContext::setup_init();

    let repo = ctx.local_repo;
    commit(repo.workdir().unwrap(), "file.txt", "content");

    let upstream_branch = branch_from_head(&repo, "upstream-branch");
    let upstream_ref = upstream_branch.into_reference();

    set_upstream(&repo, Some(&upstream_ref)).unwrap();

    let actual_merge = get_branch_merge_str(&repo);
    let actual_remote = get_branch_remote_str(&repo);

    assert_eq!(actual_merge, upstream_ref.name().unwrap());
    assert_eq!(actual_remote, ".");
}
