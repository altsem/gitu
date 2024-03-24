use git2::Repository;

use crate::Res;

pub(crate) fn set_upstream(repo: &Repository, upstream: Option<&git2::Reference>) -> Res<()> {
    let head = repo.head()?;
    let branch = head
        .shorthand()
        .ok_or("Head branch name was not valid UTF-8")?;
    let merge_cfg = format!("branch.{branch}.merge");
    let remote_cfg = format!("branch.{branch}.remote");
    let mut config = repo.config()?;

    if let Some(upstream) = upstream {
        let upstream_ref = upstream
            .name()
            .ok_or("Upstream reference name was not valid UTF-8")?;
        let upstream_name = upstream
            .shorthand()
            .ok_or("Upstream branch name was not valid UTF-8")?;
        if upstream.is_remote() {
            let remote = repo.branch_remote_name(upstream_ref)?;
            let remote = remote.as_str().ok_or("Remote name was not valid UTF-8")?;
            let upstream_name = upstream_name
                .strip_prefix(&format!("{remote}/"))
                .unwrap_or(upstream_name);

            config.set_str(&merge_cfg, &format!("refs/heads/{upstream_name}"))?;
            config.set_str(&remote_cfg, &remote)?;
        } else if upstream.is_branch() {
            config.set_str(&merge_cfg, &upstream_ref)?;
            config.set_str(&remote_cfg, ".")?;
        }
    } else {
        config.set_str(&merge_cfg, "")?;
        config.set_str(&remote_cfg, "")?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::helpers::*;

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
}
