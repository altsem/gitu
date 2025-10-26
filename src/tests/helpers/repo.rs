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
macro_rules! repo_setup_clone {
    () => {{ RepoTestContext::setup_clone(function_name!()) }};
}

impl RepoTestContext {
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

        let url = format!("file://{}", remote_dir.to_str().unwrap());
        run(&local_dir, &["git", "clone", &url, "."]);
        set_config(&local_dir);

        let local_repo = open_repo(&local_dir);
        assert_local_test_repo(&local_dir);
        assert_remote_test_repo(&remote_dir);

        Self {
            local_repo,
            dir: local_dir,
            remote_dir,
        }
    }
}

/// Just to make sure we're not accidentally modifying gitu's repo
fn assert_local_test_repo(dir: &Path) {
    assert_eq!(
        run(dir, &["git", "log", "--oneline", "--graph", "--all"]),
        "* b66a0bf add initial-file\n"
    );
    assert!(run(dir, &["git", "remote", "get-url", "origin"]).contains("gitu/testfiles"));
}

fn assert_remote_test_repo(dir: &Path) {
    assert_eq!(
        run(dir, &["git", "log", "--oneline", "--graph", "--all"]),
        "* b66a0bf add initial-file\n"
    );
    assert_eq!(run(dir, &["git", "remote"]), "");
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

    if !output.status.success() {
        let stderr = String::from_utf8(output.stderr).unwrap();
        panic!("failed to execute {:?}. Output: {}", cmd, stderr)
    }

    String::from_utf8(output.stdout).unwrap()
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
