pub mod cli;
mod git;
mod items;
mod keybinds;
mod screen;
mod theme;
mod ui;
mod util;

use crossterm::event::{self, Event, KeyEventKind};
use items::{Item, TargetData};
use keybinds::{Op, SubmenuOp, TargetOp};
use ratatui::{prelude::*, Terminal};
use screen::Screen;
use std::{
    error::Error,
    path::PathBuf,
    process::{Command, Output, Stdio},
};

type Res<T> = Result<T, Box<dyn Error>>;

#[derive(Clone)]
struct Config {
    dir: PathBuf,
}

pub(crate) struct CmdMeta {
    pub(crate) args: Vec<String>,
    pub(crate) out: Option<Output>,
}

struct State {
    config: Config,
    quit: bool,
    screens: Vec<Screen>,
    pending_submenu_op: SubmenuOp,
    pub(crate) cmd_meta: Option<CmdMeta>,
}

impl State {
    fn create(config: Config, size: Rect, args: cli::Args) -> Res<Self> {
        let screens = match args.command {
            Some(cli::Commands::Show { reference }) => {
                vec![screen::show::create(&config, size, reference)?]
            }
            Some(cli::Commands::Log { git_log_args }) => {
                vec![screen::log::create(&config, size, git_log_args)?]
            }
            Some(cli::Commands::Diff { git_diff_args }) => {
                vec![screen::diff::create(&config, size, git_diff_args)?]
            }
            None => vec![screen::status::create(&config, size)?],
        };

        Ok(Self {
            config,
            quit: args.exit_immediately,
            screens,
            pending_submenu_op: SubmenuOp::None,
            cmd_meta: None,
        })
    }

    fn screen_mut(&mut self) -> &mut Screen {
        self.screens.last_mut().expect("No screen")
    }

    fn screen(&self) -> &Screen {
        self.screens.last().expect("No screen")
    }

    pub(crate) fn issue_command<B: Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
        input: &[u8],
        mut cmd: Command,
    ) -> Res<()> {
        cmd.current_dir(&self.config.dir);

        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        self.cmd_meta = Some(CmdMeta {
            args: command_args(&cmd),
            out: None,
        });

        terminal.draw(|frame| ui::ui::<B>(frame, &*self))?;

        let mut child = cmd.spawn()?;

        use std::io::Write;
        child.stdin.take().unwrap().write_all(input)?;

        self.cmd_meta.as_mut().unwrap().out = Some(child.wait_with_output()?);
        self.screen_mut().update()?;

        Ok(())
    }

    pub(crate) fn issue_subscreen_command<B: Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
        mut cmd: Command,
    ) -> Res<()> {
        cmd.current_dir(&self.config.dir);

        cmd.stdin(Stdio::piped());
        let child = cmd.spawn()?;

        let out = child.wait_with_output()?;

        self.cmd_meta = Some(CmdMeta {
            args: command_args(&cmd),
            out: Some(out),
        });

        terminal.hide_cursor()?;
        terminal.clear()?;

        self.screen_mut().update()?;

        Ok(())
    }
}

fn command_args(cmd: &Command) -> Vec<String> {
    let mut args = vec![];
    args.push(cmd.get_program().to_string_lossy().to_string());
    args.extend(cmd.get_args().map(|arg| arg.to_string_lossy().to_string()));
    args
}

pub fn run<B: Backend>(args: cli::Args, terminal: &mut Terminal<B>) -> Result<(), Box<dyn Error>> {
    let mut state = State::create(
        Config {
            dir: String::from_utf8(
                Command::new("git")
                    .args(["rev-parse", "--show-toplevel"])
                    .output()?
                    .stdout,
            )?
            .trim_end()
            .into(),
        },
        terminal.size()?,
        args,
    )?;
    terminal.draw(|frame| ui::ui::<B>(frame, &state))?;

    while !state.quit {
        // TODO Gather all events, no need to draw for every
        if !event::poll(std::time::Duration::from_millis(u64::MAX))? {
            continue;
        }

        let event = event::read()?;
        update(terminal, &mut state, &[event])?;
    }

    Ok(())
}

pub(crate) fn update<B: Backend>(
    terminal: &mut Terminal<B>,
    state: &mut State,
    events: &[Event],
) -> Res<()> {
    for event in events {
        match *event {
            Event::Resize(w, h) => {
                for screen in state.screens.iter_mut() {
                    screen.size = Rect::new(0, 0, w, h);
                }
            }
            Event::Key(key) => {
                if key.kind == KeyEventKind::Press {
                    state.cmd_meta = None;

                    handle_op(terminal, state, key)?;
                }
            }
            _ => (),
        }
    }

    if let Some(screen) = state.screens.last_mut() {
        screen.clamp_cursor();
        terminal.draw(|frame| ui::ui::<B>(frame, &*state))?;
    }

    Ok(())
}

fn handle_op<B: Backend>(
    terminal: &mut Terminal<B>,
    state: &mut State,
    key: event::KeyEvent,
) -> Res<()> {
    let pending = if state.pending_submenu_op == SubmenuOp::Help {
        SubmenuOp::None
    } else {
        state.pending_submenu_op
    };

    if let Some(op) = keybinds::op_of_key_event(pending, key) {
        use Op::*;
        let was_submenu = state.pending_submenu_op != SubmenuOp::None;
        state.pending_submenu_op = SubmenuOp::None;

        match op {
            Quit => {
                if was_submenu {
                    // Do nothing, already cleared
                } else {
                    state.screens.pop();
                    if let Some(screen) = state.screens.last_mut() {
                        screen.update()?;
                    } else {
                        state.quit = true
                    }
                }
            }
            Refresh => state.screen_mut().update()?,
            ToggleSection => state.screen_mut().toggle_section(),
            SelectPrevious => state.screen_mut().select_previous(),
            SelectNext => state.screen_mut().select_next(),
            HalfPageUp => state.screen_mut().scroll_half_page_up(),
            HalfPageDown => state.screen_mut().scroll_half_page_down(),
            Commit => {
                state.issue_subscreen_command(terminal, git::commit_cmd())?;
            }
            CommitAmend => {
                state.issue_subscreen_command(terminal, git::commit_amend_cmd())?;
            }
            Submenu(op) => state.pending_submenu_op = op,
            LogCurrent => goto_log_screen(&state.config, &mut state.screens),
            FetchAll => {
                state.issue_command(terminal, &[], git::fetch_all_cmd())?;
            }
            PullRemote => state.issue_command(terminal, &[], git::pull_cmd())?,
            PushRemote => state.issue_command(terminal, &[], git::push_cmd())?,
            Target(target_op) => {
                if let Some(act) = &state.screen_mut().get_selected_item().target_data.clone() {
                    if let Some(mut closure) = closure_by_target_op(act, &target_op) {
                        closure(terminal, state)?;
                    }
                }
            }
            RebaseAbort => {
                state.issue_command(terminal, &[], git::rebase_abort_cmd())?;
            }
            RebaseContinue => {
                state.issue_command(terminal, &[], git::rebase_continue_cmd())?;
            }
            ShowRefs => goto_refs_screen(&state.config, &mut state.screens),
        }
    }

    Ok(())
}

pub(crate) fn list_target_ops<B: Backend>(
    target: &TargetData,
) -> impl Iterator<Item = &'static TargetOp> + '_ {
    TargetOp::list_all().filter(|target_op| closure_by_target_op::<B>(target, target_op).is_some())
}

type OpClosure<'a, B> = Box<dyn FnMut(&mut Terminal<B>, &mut State) -> Res<()> + 'a>;

/// Retrieves the 'implementation' of a `TargetOp`.
/// These are `Option<OpClosure>`s, so that the mappings
/// can be introspected.
pub(crate) fn closure_by_target_op<'a, B: Backend>(
    target: &'a TargetData,
    target_op: &TargetOp,
) -> Option<OpClosure<'a, B>> {
    use TargetData::*;
    use TargetOp::*;

    match (target_op, target) {
        (Show, Ref(r)) => goto_show_screen(r.clone()),
        (Show, File(u)) => editor(u.clone(), None),
        (Show, Delta(d)) => editor(d.new_file.clone(), None),
        (Show, Hunk(h)) => editor(h.new_file.clone(), Some(h.first_diff_line())),
        (Stage, Ref(_)) => None,
        (Stage, File(u)) => cmd_arg(git::stage_file_cmd, u),
        (Stage, Delta(d)) => cmd_arg(git::stage_file_cmd, &d.new_file),
        (Stage, Hunk(h)) => cmd(h.format_patch().into_bytes(), git::stage_patch_cmd),
        (Unstage, Ref(_)) => None,
        (Unstage, File(_)) => None,
        (Unstage, Delta(d)) => cmd_arg(git::unstage_file_cmd, &d.new_file),
        (Unstage, Hunk(h)) => cmd(h.format_patch().into_bytes(), git::unstage_patch_cmd),
        (RebaseInteractive, Ref(r)) => subscreen_arg(git::rebase_interactive_cmd, r),
        (RebaseInteractive, _) => None,
        (CommitFixup, Ref(r)) => subscreen_arg(git::commit_fixup_cmd, r),
        (CommitFixup, _) => None,
        (RebaseAutosquash, Ref(r)) => subscreen_arg(git::rebase_autosquash_cmd, r),
        (RebaseAutosquash, _) => None,
        (Discard, Ref(_)) => None,
        (Discard, File(f)) => Some(Box::new(|_term, state| {
            let path = PathBuf::from_iter([state.config.dir.to_path_buf(), f.clone().into()]);
            std::fs::remove_file(path)?;
            state.screen_mut().update()
        })),
        (Discard, Delta(d)) => {
            if d.old_file == d.new_file {
                cmd_arg(git::checkout_file_cmd, &d.old_file)
            } else {
                // TODO Discard file move
                None
            }
        }
        (Discard, Hunk(h)) => cmd(
            h.format_patch().into_bytes(),
            git::discard_unstaged_patch_cmd,
        ),
        (Checkout, Ref(r)) => cmd_arg(git::checkout_ref_cmd, r),
        (Checkout, _) => None,
    }
}

fn goto_show_screen<'a, B: Backend>(r: String) -> Option<OpClosure<'a, B>> {
    Some(Box::new(move |terminal, state| {
        state.screens.push(
            screen::show::create(&state.config, terminal.size()?, r.clone())
                .expect("Couldn't create screen"),
        );
        Ok(())
    }))
}

fn editor<'a, B: Backend>(file: String, line: Option<u32>) -> Option<OpClosure<'a, B>> {
    Some(Box::new(move |terminal, state| {
        let file: &str = &file;
        let editor = std::env::var("EDITOR").expect("EDITOR not set");
        let mut cmd = Command::new(editor.clone());
        let args = match line {
            Some(line) => match editor.as_str() {
                "vi" | "vim" | "nvim" | "nano" => {
                    vec![format!("+{}", line), file.to_string()]
                }
                _ => vec![format!("{}:{}", file, line)],
            },
            None => vec![file.to_string()],
        };

        cmd.args(args);

        state
            .issue_subscreen_command(terminal, cmd)
            .expect("Error opening editor");

        state.screen_mut().update()
    }))
}

fn cmd<'a, B: Backend>(input: Vec<u8>, command: fn() -> Command) -> Option<OpClosure<'a, B>> {
    Some(Box::new(move |terminal, state| {
        state.issue_command(terminal, &input, command())
    }))
}

fn cmd_arg<B: Backend>(command: fn(&str) -> Command, arg: &str) -> Option<OpClosure<B>> {
    Some(Box::new(move |terminal, state| {
        state.issue_command(terminal, &[], command(arg))
    }))
}

fn subscreen_arg<B: Backend>(command: fn(&str) -> Command, arg: &str) -> Option<OpClosure<B>> {
    Some(Box::new(move |terminal, state| {
        state.issue_subscreen_command(terminal, command(arg))
    }))
}

fn goto_log_screen(config: &Config, screens: &mut Vec<Screen>) {
    screens.drain(1..);
    let size = screens.last().unwrap().size;
    screens.push(screen::log::create(config, size, vec![]).expect("Couldn't create screen"));
}

fn goto_refs_screen(config: &Config, screens: &mut Vec<Screen>) {
    screens.drain(1..);
    let size = screens.last().unwrap().size;
    screens.push(screen::show_refs::create(config, size).expect("Couldn't create screen"));
}

#[cfg(test)]
#[serial_test::serial]
mod tests {
    use crate::{cli::Args, update, State};
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
    use ratatui::{backend::TestBackend, prelude::Rect, Terminal};
    use std::{env, fs, process::Command};
    use temp_dir::TempDir;

    #[test]
    fn no_repo() {
        let (ref mut terminal, _state, dir) = setup(60, 20);
        insta::assert_snapshot!(redact_hashes(terminal, dir));
    }

    #[test]
    fn help_menu() {
        let (ref mut terminal, ref mut state, dir) = setup(60, 20);
        update(terminal, state, &[key('h')]).unwrap();
        insta::assert_snapshot!(redact_hashes(terminal, dir));
    }

    #[test]
    fn fresh_init() {
        let (ref mut terminal, ref mut state, dir) = setup(60, 20);
        update(terminal, state, &[key('g')]).unwrap();
        insta::assert_snapshot!(redact_hashes(terminal, dir));
    }

    #[test]
    fn new_file() {
        let (ref mut terminal, ref mut state, dir) = setup(60, 20);
        run(&dir, &["touch", "new-file"]);
        update(terminal, state, &[key('g')]).unwrap();
        insta::assert_snapshot!(redact_hashes(terminal, dir));
    }

    #[test]
    fn unstaged_changes() {
        let (ref mut terminal, ref mut state, dir) = setup(60, 20);
        commit(&dir, "testfile", "testing\ntesttest");
        fs::write(dir.child("testfile"), "test\ntesttest").expect("error writing to file");

        update(terminal, state, &[key('g')]).unwrap();
        insta::assert_snapshot!(redact_hashes(terminal, dir));
    }

    #[test]
    fn staged_file() {
        let (ref mut terminal, ref mut state, dir) = setup(60, 20);
        run(&dir, &["touch", "new-file"]);
        run(&dir, &["git", "add", "new-file"]);
        update(terminal, state, &[key('g')]).unwrap();
        insta::assert_snapshot!(redact_hashes(terminal, dir));
    }

    #[test]
    fn log() {
        let (ref mut terminal, ref mut state, dir) = setup(60, 20);
        commit(&dir, "firstfile", "testing\ntesttest\n");
        commit(&dir, "secondfile", "testing\ntesttest\n");
        update(terminal, state, &[key('g'), key('l'), key('l')]).unwrap();
        insta::assert_snapshot!(redact_hashes(terminal, dir));
    }

    #[test]
    fn show() {
        let (ref mut terminal, ref mut state, dir) = setup(60, 20);
        commit(&dir, "firstfile", "This should not be visible\n");
        commit(&dir, "secondfile", "This should be visible\n");
        update(
            terminal,
            state,
            &[key('g'), key('l'), key('l'), key_code(KeyCode::Enter)],
        )
        .unwrap();
        insta::assert_snapshot!(redact_hashes(terminal, dir));
    }

    #[test]
    fn rebase_conflict() {
        let (ref mut terminal, ref mut state, dir) = setup(60, 20);
        commit(&dir, "new-file", "hello");

        run(&dir, &["git", "checkout", "-b", "other-branch"]);
        commit(&dir, "new-file", "hey");

        run(&dir, &["git", "checkout", "master"]);
        commit(&dir, "new-file", "hi");

        run(&dir, &["git", "checkout", "other-branch"]);
        run(&dir, &["git", "rebase", "master"]);

        update(terminal, state, &[key('g')]).unwrap();
        insta::assert_snapshot!(redact_hashes(terminal, dir));
    }

    #[test]
    fn merge_conflict() {
        let (ref mut terminal, ref mut state, dir) = setup(60, 20);
        commit(&dir, "new-file", "hello");

        run(&dir, &["git", "checkout", "-b", "other-branch"]);
        commit(&dir, "new-file", "hey");

        run(&dir, &["git", "checkout", "master"]);
        commit(&dir, "new-file", "hi");

        run(&dir, &["git", "merge", "other-branch"]);

        update(terminal, state, &[key('g')]).unwrap();
        insta::assert_snapshot!(redact_hashes(terminal, dir));
    }

    fn setup(width: u16, height: u16) -> (Terminal<TestBackend>, State, TempDir) {
        let terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        let dir = TempDir::new().unwrap();
        env::set_var("GIT_DIR", dir.child(".git"));

        run(&dir, &["git", "init", "--initial-branch", "master"]);
        run(&dir, &["git", "config", "user.email", "ci@example.com"]);
        run(&dir, &["git", "config", "user.name", "CI"]);

        let state = State::create(
            crate::Config {
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

        (terminal, state, dir)
    }

    fn commit(dir: &TempDir, file_name: &str, contents: &str) {
        let path = dir.child(file_name);
        let message = match path.try_exists() {
            Ok(true) => format!("modify {}\n\nCommit body goes here\n", file_name),
            _ => format!("add {}\n\nCommit body goes here\n", file_name),
        };
        fs::write(path, contents).expect("error writing to file");
        run(dir, &["git", "add", file_name]);
        run(dir, &["git", "commit", "-m", &message]);
    }

    fn run(dir: &TempDir, cmd: &[&str]) -> String {
        String::from_utf8(
            Command::new(cmd[0])
                .args(&cmd[1..])
                .env("GIT_COMMITTER_DATE", "Sun Feb 18 14:00 2024 +0100")
                .current_dir(dir.path())
                .output()
                .unwrap_or_else(|_| panic!("failed to execute {:?}", cmd))
                .stdout,
        )
        .expect("failed converting output to String")
    }

    fn key(char: char) -> Event {
        Event::Key(KeyEvent::new(KeyCode::Char(char), KeyModifiers::empty()))
    }

    fn key_code(code: KeyCode) -> Event {
        Event::Key(KeyEvent::new(code, KeyModifiers::empty()))
    }

    fn redact_hashes(terminal: &mut Terminal<TestBackend>, dir: TempDir) -> String {
        let mut debug_output = format!("{:#?}", terminal.backend().buffer());

        for hash in run(&dir, &["git", "log", "--all", "--format=%H", "HEAD"]).lines() {
            debug_output = debug_output.replace(hash, &"_".repeat(hash.len()));
        }
        for hash in run(&dir, &["git", "log", "--all", "--format=%h", "HEAD"]).lines() {
            debug_output = debug_output.replace(hash, &"_".repeat(hash.len()));
        }

        debug_output
    }
}
