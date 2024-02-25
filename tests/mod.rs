use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use git2::Repository;
use gitu::{cli::Args, update, State};
use ratatui::{backend::TestBackend, prelude::Rect, Terminal};
use std::{env, fs, path::Path, process::Command};
use temp_dir::TempDir;

#[test]
fn no_repo() {
    let ctx = TestContext::setup_init(60, 20);
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn help_menu() {
    let mut ctx = TestContext::setup_init(60, 20);
    ctx.update(&[key('h')]);
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn fresh_init() {
    let mut ctx = TestContext::setup_init(60, 20);
    ctx.update(&[key('g')]);
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn new_file() {
    let mut ctx = TestContext::setup_init(60, 20);
    run(ctx.dir.path(), &["touch", "new-file"]);
    ctx.update(&[key('g')]);
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn unstaged_changes() {
    let mut ctx = TestContext::setup_init(60, 20);
    commit(ctx.dir.path(), "testfile", "testing\ntesttest");
    fs::write(ctx.dir.child("testfile"), "test\ntesttest").expect("error writing to file");

    ctx.update(&[key('g')]);
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn staged_file() {
    let mut ctx = TestContext::setup_init(60, 20);
    run(ctx.dir.path(), &["touch", "new-file"]);
    run(ctx.dir.path(), &["git", "add", "new-file"]);
    ctx.update(&[key('g')]);
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn log() {
    let mut ctx = TestContext::setup_clone(60, 20);
    commit(ctx.dir.path(), "firstfile", "testing\ntesttest\n");
    commit(ctx.dir.path(), "secondfile", "testing\ntesttest\n");
    ctx.update(&[key('g'), key('l'), key('l')]);
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn show() {
    let mut ctx = TestContext::setup_clone(60, 20);
    commit(ctx.dir.path(), "firstfile", "This should be visible\n");
    ctx.update(&[key('g'), key('l'), key('l'), key_code(KeyCode::Enter)]);
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn rebase_conflict() {
    let mut ctx = TestContext::setup_clone(60, 20);
    commit(ctx.dir.path(), "new-file", "hello");

    run(ctx.dir.path(), &["git", "checkout", "-b", "other-branch"]);
    commit(ctx.dir.path(), "new-file", "hey");

    run(ctx.dir.path(), &["git", "checkout", "main"]);
    commit(ctx.dir.path(), "new-file", "hi");

    run(ctx.dir.path(), &["git", "checkout", "other-branch"]);
    run(ctx.dir.path(), &["git", "rebase", "main"]);

    ctx.update(&[key('g')]);
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn merge_conflict() {
    let mut ctx = TestContext::setup_clone(60, 20);
    commit(ctx.dir.path(), "new-file", "hello");

    run(ctx.dir.path(), &["git", "checkout", "-b", "other-branch"]);
    commit(ctx.dir.path(), "new-file", "hey");

    run(ctx.dir.path(), &["git", "checkout", "main"]);
    commit(ctx.dir.path(), "new-file", "hi");

    run(ctx.dir.path(), &["git", "merge", "other-branch"]);

    ctx.update(&[key('g')]);
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn moved_file() {
    let mut ctx = TestContext::setup_clone(60, 20);
    commit(ctx.dir.path(), "new-file", "hello");
    run(ctx.dir.path(), &["git", "mv", "new-file", "moved-file"]);

    ctx.update(&[key('g')]);
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn hide_untracked() {
    let mut ctx = TestContext::setup_clone(60, 10);
    let mut config = ctx.state.repo.config().unwrap();
    config.set_str("status.showUntrackedFiles", "off").unwrap();
    run(ctx.dir.path(), &["touch", "i-am-untracked"]);

    ctx.update(&[key('g')]);
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn new_commit() {
    let mut ctx = TestContext::setup_clone(60, 10);
    commit(ctx.dir.path(), "new-file", "");

    ctx.update(&[key('g')]);
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn push() {
    let mut ctx = TestContext::setup_clone(60, 10);
    commit(ctx.dir.path(), "new-file", "");

    ctx.update(&[key('P'), key('p')]);
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn fetch_all() {
    let mut ctx = TestContext::setup_clone(60, 10);
    clone_and_commit(&ctx.remote_dir, "remote-file", "hello");

    ctx.update(&[key('f'), key('a')]);
    insta::assert_snapshot!(ctx.redact_buffer());
}

#[test]
fn pull() {
    let mut ctx = TestContext::setup_clone(60, 10);
    clone_and_commit(&ctx.remote_dir, "remote-file", "hello");

    ctx.update(&[key('F'), key('p')]);
    insta::assert_snapshot!(ctx.redact_buffer());
}

struct TestContext {
    terminal: Terminal<TestBackend>,
    state: State,
    dir: TempDir,
    remote_dir: TempDir,
}

impl TestContext {
    fn setup_init(width: u16, height: u16) -> Self {
        let terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        let remote_dir = TempDir::new().unwrap();
        let dir = TempDir::new().unwrap();

        set_env_vars();
        run(dir.path(), &["git", "init", "--initial-branch=main"]);

        let state = State::create(
            Repository::open(dir.path()).unwrap(),
            gitu::Config {
                dir: dir.path().into(),
            },
            Rect::new(0, 0, width, height),
            Args {
                command: None,
                status: false,
                exit_immediately: false,
            },
        )
        .unwrap();

        set_config(state.repo.config().unwrap());

        Self {
            terminal,
            state,
            dir,
            remote_dir,
        }
    }

    fn setup_clone(width: u16, height: u16) -> Self {
        let terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        let remote_dir = TempDir::new().unwrap();
        let dir = TempDir::new().unwrap();

        set_env_vars();
        run(
            remote_dir.path(),
            &["git", "init", "--bare", "--initial-branch=main"],
        );
        clone_and_commit(&remote_dir, "initial-file", "hello");
        run(
            dir.path(),
            &["git", "clone", remote_dir.path().to_str().unwrap(), "."],
        );

        let state = State::create(
            Repository::open(dir.path()).unwrap(),
            gitu::Config {
                dir: dir.path().into(),
            },
            Rect::new(0, 0, width, height),
            Args {
                command: None,
                status: false,
                exit_immediately: false,
            },
        )
        .unwrap();

        set_config(state.repo.config().unwrap());

        Self {
            terminal,
            state,
            dir,
            remote_dir,
        }
    }

    fn update(&mut self, events: &[Event]) {
        update(&mut self.terminal, &mut self.state, events).unwrap();
    }

    fn redact_buffer(&self) -> String {
        let mut debug_output = format!("{:#?}", self.terminal.backend().buffer());

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

fn clone_and_commit(remote_dir: &TempDir, file_name: &str, file_content: &str) {
    let other_dir = TempDir::new().unwrap();

    run(
        other_dir.path(),
        &["git", "clone", remote_dir.path().to_str().unwrap(), "."],
    );

    let other_repo = Repository::open(other_dir.path()).unwrap();
    set_config(other_repo.config().unwrap());

    commit(other_dir.path(), file_name, file_content);
    run(other_dir.path(), &["git", "push"]);
}

fn set_env_vars() {
    env::set_var("GIT_CONFIG_GLOBAL", "/dev/null");
    env::set_var("GIT_CONFIG_SYSTEM", "/dev/null");
    env::set_var("GIT_COMMITTER_DATE", "Sun Feb 18 14:00 2024 +0100");
}

fn set_config(mut config: git2::Config) {
    config.set_str("user.email", "ci@example.com").unwrap();
    config.set_str("user.name", "CI").unwrap();
}

fn commit(dir: &Path, file_name: &str, contents: &str) {
    let mut path = dir.to_path_buf();
    path.push(file_name);
    let message = match path.try_exists() {
        Ok(true) => format!("modify {}\n\nCommit body goes here\n", file_name),
        _ => format!("add {}\n\nCommit body goes here\n", file_name),
    };
    fs::write(path, contents).expect("error writing to file");
    run(dir, &["git", "add", file_name]);
    run(dir, &["git", "commit", "-m", &message]);
}

fn run(dir: &Path, cmd: &[&str]) {
    Command::new(cmd[0])
        .args(&cmd[1..])
        .current_dir(dir)
        .output()
        .unwrap_or_else(|_| panic!("failed to execute {:?}", cmd));
}

fn key(char: char) -> Event {
    let mods = if char.is_uppercase() {
        KeyModifiers::SHIFT
    } else {
        KeyModifiers::empty()
    };

    Event::Key(KeyEvent::new(KeyCode::Char(char), mods))
}

fn key_code(code: KeyCode) -> Event {
    Event::Key(KeyEvent::new(code, KeyModifiers::empty()))
}
