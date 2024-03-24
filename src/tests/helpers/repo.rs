use std::{env, fs, path::Path, process::Command};

use temp_dir::TempDir;

pub struct RepoTestContext {
    pub dir: TempDir,
    pub remote_dir: TempDir,
    pub local_repo: git2::Repository,
    pub remote_repo: git2::Repository,
}

fn open_repo(dir: &TempDir) -> git2::Repository {
    git2::Repository::open(dir.path().to_path_buf()).unwrap()
}

impl RepoTestContext {
    pub fn setup_init() -> Self {
        let remote_dir = TempDir::new().unwrap();
        let dir = TempDir::new().unwrap();

        set_env_vars();
        run(dir.path(), &["git", "init", "--initial-branch=main"]);
        set_config(dir.path());

        Self {
            local_repo: open_repo(&dir),
            remote_repo: open_repo(&remote_dir),
            dir,
            remote_dir,
        }
    }

    pub fn setup_clone() -> Self {
        let remote_dir = TempDir::new().unwrap();
        let dir = TempDir::new().unwrap();

        set_env_vars();

        run(
            remote_dir.path(),
            &["git", "init", "--bare", "--initial-branch=main"],
        );
        set_config(remote_dir.path());

        clone_and_commit(&remote_dir, "initial-file", "hello");
        run(
            dir.path(),
            &["git", "clone", remote_dir.path().to_str().unwrap(), "."],
        );
        set_config(dir.path());

        Self {
            local_repo: open_repo(&dir),
            remote_repo: open_repo(&remote_dir),
            dir,
            remote_dir,
        }
    }
}

pub fn set_env_vars() {
    // https://git-scm.com/book/en/v2/Git-Internals-Environment-Variables
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

pub fn run(dir: &Path, cmd: &[&str]) -> String {
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

fn set_config(path: &Path) {
    run(path, &["git", "config", "user.email", "ci@example.com"]);
    run(path, &["git", "config", "user.name", "CI"]);
}

pub fn clone_and_commit(remote_dir: &TempDir, file_name: &str, file_content: &str) {
    let other_dir = TempDir::new().unwrap();

    run(
        other_dir.path(),
        &["git", "clone", remote_dir.path().to_str().unwrap(), "."],
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
