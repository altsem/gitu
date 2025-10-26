use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

use temp_dir::TempDir;

pub struct RepoTestContext {
    pub dir: PathBuf,
    pub remote_dir: PathBuf,
    pub local_repo: git2::Repository,
}

fn open_repo(dir: &Path) -> git2::Repository {
    git2::Repository::open(dir).unwrap()
}

#[macro_export]
macro_rules! repo_setup_init {
    () => {{ RepoTestContext::setup_init(function_name!()) }};
}

#[macro_export]
macro_rules! repo_setup_clone {
    () => {{ RepoTestContext::setup_clone(function_name!()) }};
}

impl RepoTestContext {
    pub fn setup_init(test_name: &str) -> Self {
        fs::create_dir_all("testfiles").unwrap();
        let testfiles = fs::canonicalize("testfiles").unwrap();
        let dir = &testfiles.join(test_name.replace(":", "_"));
        fs::create_dir_all(dir).unwrap();
        fs::remove_dir_all(dir).unwrap();

        let local_dir = dir.join("local");
        fs::create_dir_all(&local_dir).unwrap();
        let remote_dir = dir.join("remote");
        fs::create_dir_all(&remote_dir).unwrap();

        set_env_vars();
        run(&local_dir, &["git", "init", "--initial-branch=main"]);
        run(&remote_dir, &["git", "init", "--initial-branch=main"]);
        set_config(&local_dir);
        set_config(&remote_dir);

        let local_repo = open_repo(&local_dir);
        assert_repo_commit_count(&local_dir, 0);
        assert_repo_commit_count(&remote_dir, 0);

        Self {
            local_repo,
            dir: local_dir,
            remote_dir,
        }
    }

    pub fn setup_clone(test_name: &str) -> Self {
        fs::create_dir_all("testfiles").unwrap();
        let testfiles = fs::canonicalize("testfiles").unwrap();
        let dir = &testfiles.join(test_name.replace(":", "_"));
        fs::create_dir_all(dir).unwrap();
        fs::remove_dir_all(dir).unwrap();

        let local_dir = dir.join("local");
        fs::create_dir_all(&local_dir).unwrap();
        let remote_dir = dir.join("remote");
        fs::create_dir_all(&remote_dir).unwrap();

        set_env_vars();
        run(
            &remote_dir,
            &["git", "init", "--bare", "--initial-branch=main"],
        );
        set_config(&remote_dir);
        clone_and_commit(&remote_dir, "initial-file", "hello");

        run(
            &local_dir,
            &["git", "clone", remote_dir.to_str().unwrap(), "."],
        );
        set_config(&local_dir);

        let local_repo = open_repo(&local_dir);
        assert_eq!(local_repo.revwalk().unwrap().count(), 0);
        assert_repo_commit_count(&local_dir, 1);
        assert_repo_commit_count(&remote_dir, 1);

        Self {
            local_repo,
            dir: local_dir,
            remote_dir,
        }
    }
}

/// Just to make sure we're not accidentally modifying gitu's repo
fn assert_repo_commit_count(remote_dir: &Path, expected_commit_count: usize) {
    let repo = open_repo(remote_dir);
    let mut revwalk = repo.revwalk().unwrap();
    if revwalk.push_head().is_ok() {
        assert_eq!(revwalk.count(), expected_commit_count);
    } else {
        assert!(repo.is_empty().unwrap());
    }
}

pub fn set_env_vars() {
    // https://git-scm.com/book/en/v2/Git-Internals-Environment-Variables
    unsafe {
        env::set_var("GIT_CONFIG_GLOBAL", "/dev/null");
        env::set_var("GIT_CONFIG_SYSTEM", "/dev/null");
        env::set_var("GIT_AUTHOR_NAME", "Author Name");
        env::set_var("GIT_AUTHOR_EMAIL", "author@email.com");
        env::set_var("GIT_AUTHOR_DATE", "Fri Feb 16 11:11 2024 +0100");
        env::set_var("GIT_COMMITTER_NAME", "Committer Name");
        env::set_var("GIT_COMMITTER_EMAIL", "committer@email.com");
        env::set_var("GIT_COMMITTER_DATE", "Sun Feb 18 14:00 2024 +0100");
        env::set_var("LC_ALL", "C");
    }
}

pub fn run_ignore_status(dir: &Path, cmd: &[&str]) -> String {
    String::from_utf8(
        Command::new(cmd[0])
            .args(&cmd[1..])
            .current_dir(dir)
            .output()
            .unwrap_or_else(|_| panic!("failed to execute {:?}", cmd))
            .stderr,
    )
    .unwrap()
}

pub fn run(dir: &Path, cmd: &[&str]) -> String {
    let output = Command::new(cmd[0])
        .args(&cmd[1..])
        .current_dir(dir)
        .output()
        .unwrap_or_else(|_| panic!("failed to execute {:?}", cmd));

    let stderr = String::from_utf8(output.stderr).unwrap();
    if !output.status.success() {
        panic!("failed to execute {:?}. Output: {}", cmd, stderr)
    }

    stderr
}

fn set_config(path: &Path) {
    run(path, &["git", "config", "user.email", "ci@example.com"]);
    run(path, &["git", "config", "user.name", "CI"]);
}

pub fn clone_and_commit(remote_dir: &Path, file_name: &str, file_content: &str) {
    let other_dir = TempDir::new().unwrap();

    run(
        other_dir.path(),
        &["git", "clone", remote_dir.to_str().unwrap(), "."],
    );

    set_config(other_dir.path());

    commit(other_dir.path(), file_name, file_content);
    run(other_dir.path(), &["git", "push"]);
}

pub fn commit(dir: &Path, file_name: &str, contents: &str) {
    let path = dir.to_path_buf().join(file_name);
    let message = match path.try_exists() {
        Ok(true) => format!("modify {}\n\nCommit body goes here\n", file_name),
        _ => format!("add {}\n\nCommit body goes here\n", file_name),
    };
    fs::write(path, contents).expect("error writing to file");
    run(dir, &["git", "add", file_name]);
    run(dir, &["git", "commit", "-m", &message]);
}
