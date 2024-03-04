use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use git2::Repository;
use gitu::{cli::Args, config::Config, State};
use ratatui::{backend::TestBackend, prelude::Rect, Terminal};
use std::{env, fs, path::Path, process::Command};
use temp_dir::TempDir;

pub struct TestContext {
    pub term: Terminal<TestBackend>,
    pub dir: TempDir,
    pub remote_dir: TempDir,
    pub size: Rect,
}

impl TestContext {
    pub fn setup_init(width: u16, height: u16) -> Self {
        let term = Terminal::new(TestBackend::new(width, height)).unwrap();
        let remote_dir = TempDir::new().unwrap();
        let dir = TempDir::new().unwrap();

        set_env_vars();
        run(dir.path(), &["git", "init", "--initial-branch=main"]);
        set_config(dir.path());

        Self {
            term,
            dir,
            remote_dir,
            size: Rect::new(0, 0, width, height),
        }
    }

    pub fn setup_clone(width: u16, height: u16) -> Self {
        let term = Terminal::new(TestBackend::new(width, height)).unwrap();
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
            term,
            dir,
            remote_dir,
            size: Rect::new(0, 0, width, height),
        }
    }

    pub fn init_state(&mut self) -> State {
        let mut state = State::create(
            Repository::open(self.dir.path()).unwrap(),
            self.size,
            Args {
                command: None,
                exit_immediately: false,
            },
            Config::default(),
        )
        .unwrap();

        gitu::update(&mut self.term, &mut state, &[]).unwrap();

        state
    }

    pub fn redact_buffer(&self) -> String {
        let mut debug_output = format!("{:#?}", self.term.backend().buffer());

        [&self.dir, &self.remote_dir]
            .iter()
            .flat_map(|dir| Repository::open(dir.path()).ok())
            .for_each(|repo| {
                let mut revwalk = repo.revwalk().unwrap();
                revwalk.push_head().ok();
                revwalk
                    .flat_map(|maybe_oid| maybe_oid.and_then(|oid| repo.find_commit(oid)))
                    .for_each(|commit| {
                        let id = commit.as_object().id().to_string();
                        let short = commit.as_object().short_id().unwrap();
                        let short_id = short.as_str().unwrap();

                        debug_output = debug_output.replace(&id, &"_".repeat(id.len()));
                        debug_output = debug_output.replace(short_id, &"_".repeat(short_id.len()));
                    });
            });

        redact_temp_dir(&self.dir, &mut debug_output);
        redact_temp_dir(&self.remote_dir, &mut debug_output);

        debug_output
    }
}

fn redact_temp_dir(temp_dir: &TempDir, debug_output: &mut String) {
    let text = temp_dir.path().to_str().unwrap();
    *debug_output = debug_output.replace(text, &" ".repeat(text.len()));
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

fn set_env_vars() {
    env::set_var("GIT_CONFIG_GLOBAL", "/dev/null");
    env::set_var("GIT_CONFIG_SYSTEM", "/dev/null");
    env::set_var("GIT_COMMITTER_DATE", "Sun Feb 18 14:00 2024 +0100");
}

fn set_config(path: &Path) {
    run(path, &["git", "config", "user.email", "ci@example.com"]);
    run(path, &["git", "config", "user.name", "CI"]);
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

pub fn run(dir: &Path, cmd: &[&str]) {
    Command::new(cmd[0])
        .args(&cmd[1..])
        .current_dir(dir)
        .output()
        .unwrap_or_else(|_| panic!("failed to execute {:?}", cmd));
}

pub fn key(char: char) -> Event {
    let mods = if char.is_uppercase() {
        KeyModifiers::SHIFT
    } else {
        KeyModifiers::empty()
    };

    Event::Key(KeyEvent::new(KeyCode::Char(char), mods))
}

pub fn key_code(code: KeyCode) -> Event {
    Event::Key(KeyEvent::new(code, KeyModifiers::empty()))
}
