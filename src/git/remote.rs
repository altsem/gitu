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
