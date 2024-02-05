mod cli;
mod command;
mod diff;
mod git;
mod items;
mod keybinds;
mod screen;
mod status;
mod theme;
mod ui;
mod util;

use clap::Parser;
use command::IssuedCommand;
use crossterm::{
    event::{self, Event, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use items::{Item, TargetData};
use keybinds::{Op, TargetOp, TransientOp};
use ratatui::{prelude::*, Terminal};
use screen::Screen;
use std::{
    error::Error,
    io::{self, stderr, BufWriter},
    path::PathBuf,
    process::Command,
};

type Res<T> = Result<T, Box<dyn Error>>;

// TODO Initialize per `State`? Tests are flaky likely due to GIT_DIR here being global.
lazy_static::lazy_static! {
    static ref USE_DELTA: bool = Command::new("delta").output().map(|out| out.status.success()).unwrap_or(false);
}

struct Config {
    dir: PathBuf,
}

struct State {
    config: Config,
    quit: bool,
    screens: Vec<Screen>,
    pending_transient_op: TransientOp,
    pub(crate) command: Option<IssuedCommand>,
}

impl State {
    fn create(config: Config, args: cli::Args) -> Res<Self> {
        let screens = match args.command {
            Some(cli::Commands::Show { git_show_args }) => {
                vec![screen::show::create(&config, git_show_args)?]
            }
            Some(cli::Commands::Log { git_log_args }) => {
                vec![screen::log::create(&config, git_log_args)?]
            }
            Some(cli::Commands::Diff { git_diff_args }) => {
                vec![screen::diff::create(&config, git_diff_args)?]
            }
            None => vec![screen::status::create(&config, args.status)?],
        };

        Ok(Self {
            config,
            quit: false,
            screens,
            pending_transient_op: TransientOp::None,
            command: None,
        })
    }

    fn screen_mut(&mut self) -> &mut Screen {
        self.screens.last_mut().expect("No screen")
    }

    fn screen(&self) -> &Screen {
        self.screens.last().expect("No screen")
    }

    pub(crate) fn issue_command(
        &mut self,
        input: &[u8],
        mut command: Command,
    ) -> Result<(), io::Error> {
        command.current_dir(&self.config.dir);

        if !self.command.as_mut().is_some_and(|cmd| cmd.is_running()) {
            self.command = Some(IssuedCommand::spawn(input, command)?);
        }

        Ok(())
    }

    pub(crate) fn issue_subscreen_command<B: Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
        mut command: Command,
    ) -> Result<(), io::Error> {
        command.current_dir(&self.config.dir);

        if !self.command.as_mut().is_some_and(|cmd| cmd.is_running()) {
            self.command = Some(IssuedCommand::spawn_in_subscreen(terminal, command)?);
        }

        Ok(())
    }

    pub(crate) fn clear_finished_command(&mut self) {
        if let Some(ref mut command) = self.command {
            if !command.is_running() {
                self.command = None
            }
        }
    }

    pub(crate) fn handle_command_output(&mut self) -> Res<()> {
        if let Some(cmd) = &mut self.command {
            cmd.read_command_output_to_buffer();

            if cmd.just_finished() {
                self.screen_mut().update()?;
            }
        }
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut terminal = Terminal::new(CrosstermBackend::new(BufWriter::new(stderr())))?;
    terminal.hide_cursor()?;
    enable_raw_mode()?;
    stderr().execute(EnterAlternateScreen)?;

    run(cli::Args::parse(), &mut terminal)?;

    stderr().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

fn run<B: Backend>(args: cli::Args, terminal: &mut Terminal<B>) -> Result<(), Box<dyn Error>> {
    let mut state = State::create(
        Config {
            dir: String::from_utf8(
                Command::new("git")
                    .args(&["rev-parse", "--show-toplevel"])
                    .output()?
                    .stdout,
            )?
            .trim_end()
            .into(),
        },
        args,
    )?;
    update(terminal, &mut state, &[])?;

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
    state.handle_command_output()?;

    for event in events {
        match *event {
            Event::Resize(w, h) => state.screen_mut().size = (w, h),
            Event::Key(key) => {
                if key.kind == KeyEventKind::Press {
                    state.clear_finished_command();

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
    let pending = if state.pending_transient_op == TransientOp::Help {
        TransientOp::None
    } else {
        state.pending_transient_op
    };

    if let Some(op) = keybinds::op_of_key_event(pending, key) {
        use Op::*;
        let was_transient = state.pending_transient_op != TransientOp::None;
        state.pending_transient_op = TransientOp::None;

        match op {
            Quit => {
                if was_transient {
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
                state.screen_mut().update()?;
            }
            CommitAmend => {
                state.issue_subscreen_command(terminal, git::commit_amend_cmd())?;
                state.screen_mut().update()?;
            }
            Transient(op) => state.pending_transient_op = op,
            LogCurrent => goto_log_screen(&state.config, &mut state.screens),
            FetchAll => {
                state.issue_command(&[], git::fetch_all_cmd())?;
                state.screen_mut().update()?;
            }
            PullRemote => state.issue_command(&[], git::pull_cmd())?,
            PushRemote => state.issue_command(&[], git::push_cmd())?,
            Target(target_op) => {
                if let Some(act) = &state.screen_mut().get_selected_item().target_data.clone() {
                    if let Some(mut closure) = closure_by_target_op(act, &target_op) {
                        closure(terminal, state)?;
                    }
                }
            }
            RebaseAbort => {
                state.issue_command(&[], git::rebase_abort_cmd())?;
                state.screen_mut().update()?;
            }
            RebaseContinue => {
                state.issue_command(&[], git::rebase_continue_cmd())?;
                state.screen_mut().update()?;
            }
            ShowRefs => goto_refs_screen(&state.config, &mut state.screens),
        }
    }

    Ok(())
}

pub(crate) fn list_target_ops<'a, B: Backend>(
    target: &'a TargetData,
) -> impl Iterator<Item = &'static TargetOp> + 'a {
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
        (Stage, File(u)) => cmd_arg(git::stage_file_cmd, &u),
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
        (Checkout, Ref(r)) => cmd_arg(git::checkout_ref_cmd, &r),
        (Checkout, _) => None,
    }
}

fn goto_show_screen<'a, B: Backend>(r: String) -> Option<OpClosure<'a, B>> {
    Some(Box::new(move |_terminal, state| {
        state.screens.push(
            screen::show::create(&state.config, vec![r.clone()]).expect("Couldn't create screen"),
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
    Some(Box::new(move |_terminal, state| {
        state
            .issue_command(&input, command())
            .expect("Error unstaging hunk");
        state.screen_mut().update()
    }))
}

fn cmd_arg<B: Backend>(command: fn(&str) -> Command, arg: &String) -> Option<OpClosure<B>> {
    let arg_clone = arg.clone();
    Some(Box::new(move |_terminal, state| {
        state
            .issue_command(&[], command(&arg_clone))
            .expect("Error unstaging hunk");
        state.screen_mut().update()
    }))
}

fn subscreen_arg<B: Backend>(command: fn(&str) -> Command, arg: &String) -> Option<OpClosure<B>> {
    let arg_clone = arg.clone();
    Some(Box::new(move |terminal, state| {
        state
            .issue_subscreen_command(terminal, command(&arg_clone))
            .expect("Error issuing command");
        state.screen_mut().update()
    }))
}

fn goto_log_screen(config: &Config, screens: &mut Vec<Screen>) {
    screens.drain(1..);
    screens.push(screen::log::create(config, vec![]).expect("Couldn't create screen"));
}

fn goto_refs_screen(config: &Config, screens: &mut Vec<Screen>) {
    screens.drain(1..);
    screens.push(screen::show_refs::create(config).expect("Couldn't create screen"));
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
    use ratatui::{backend::TestBackend, Terminal};
    use temp_dir::TempDir;

    use crate::{cli::Args, update, Res, State};

    #[test]
    fn no_repo() {
        let (terminal, _state, _dir) = setup(70, 5);
        insta::assert_debug_snapshot!(terminal.backend().buffer());
    }

    #[test]
    fn help_menu() {
        let (ref mut terminal, ref mut state, _dir) = setup(70, 12);
        update(terminal, state, &[key('h')]).unwrap();
        insta::assert_debug_snapshot!(terminal.backend().buffer());
    }

    #[test]
    fn fresh_init() -> Res<()> {
        let (ref mut terminal, ref mut state, dir) = setup(70, 5);
        assert!(Command::new("git")
            .arg("init")
            .current_dir(dir.path())
            .status()?
            .success());
        update(terminal, state, &[key('g')]).unwrap();
        insta::assert_debug_snapshot!(terminal.backend().buffer());
        Ok(())
    }

    #[test]
    fn new_file() -> Res<()> {
        let (ref mut terminal, ref mut state, dir) = setup(70, 5);
        assert!(Command::new("git")
            .arg("init")
            .current_dir(dir.path())
            .status()?
            .success());
        assert!(Command::new("touch")
            .arg("new-file")
            .current_dir(dir.path())
            .status()?
            .success());
        update(terminal, state, &[key('g')]).unwrap();
        insta::assert_debug_snapshot!(terminal.backend().buffer());
        Ok(())
    }

    #[test]
    fn stage_file() -> Res<()> {
        let (ref mut terminal, ref mut state, dir) = setup(70, 5);
        assert!(Command::new("git")
            .arg("init")
            .current_dir(dir.path())
            .status()?
            .success());
        assert!(Command::new("touch")
            .arg("new-file")
            .current_dir(dir.path())
            .status()?
            .success());
        update(terminal, state, &[key('g'), key('j'), key('s'), key('g')]).unwrap();
        insta::assert_debug_snapshot!(terminal.backend().buffer());
        Ok(())
    }

    fn key(char: char) -> Event {
        Event::Key(KeyEvent::new(KeyCode::Char(char), KeyModifiers::empty()))
    }

    fn setup(width: u16, height: u16) -> (Terminal<TestBackend>, State, TempDir) {
        let terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        let dir = TempDir::new().unwrap();

        let state = State::create(
            crate::Config {
                dir: dir.path().into(),
            },
            Args {
                command: None,
                status: false,
            },
        )
        .unwrap();

        (terminal, state, dir)
    }
}
