use crate::git::remote::set_upstream;
use crate::tests::helpers::RepoTestContext;

// TODO: test that the remote is actually updated; test the different branches of `set_upstream`
#[test]
fn set_upstream_basic() {
    let ctx = RepoTestContext::setup_clone();

    let repo = ctx.local_repo;
    let head = repo.head().unwrap();
    let remote_name = repo.branch_upstream_remote(head.name().unwrap()).unwrap();
    let remote_name = remote_name.as_str().unwrap();

    let upstream_branch = ctx
        .remote_repo
        .branch(
            "upstream-branch",
            &ctx.remote_repo.head().unwrap().peel_to_commit().unwrap(),
            true,
        )
        .unwrap();
    let upstream_ref = upstream_branch.into_reference();

    repo.find_remote(remote_name)
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

    let actual_merge = repo.branch_upstream_name(head.name().unwrap()).unwrap();
    let actual_merge = actual_merge.as_str().unwrap();
    let actual_remote = repo.branch_upstream_remote(head.name().unwrap()).unwrap();
    let actual_remote = actual_remote.as_str().unwrap();

    assert_eq!(actual_merge, upstream_ref.name().unwrap());
    assert_eq!(actual_remote, remote_name);
}
