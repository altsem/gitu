mod cli;
mod command;
mod diff;
mod git;
mod items;
mod keybinds;
mod process;
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
use ratatui::prelude::CrosstermBackend;
use screen::Screen;
use std::{
    io::{self, stderr, BufWriter, Stderr},
    process::Command,
};

type Terminal = ratatui::Terminal<CrosstermBackend<BufWriter<Stderr>>>;

lazy_static::lazy_static! {
    static ref USE_DELTA: bool = Command::new("delta").output().map(|out| out.status.success()).unwrap_or(false);
    static ref GIT_DIR: String = process::run(&["git", "rev-parse", "--show-toplevel"])
            .0
            .trim_end().to_string();
}

struct State {
    quit: bool,
    screens: Vec<Screen>,
    pending_transient_op: TransientOp,
    pub(crate) command: Option<IssuedCommand>,
}

impl State {
    fn screen_mut(&mut self) -> &mut Screen {
        self.screens.last_mut().expect("No screen")
    }

    fn screen(&self) -> &Screen {
        self.screens.last().expect("No screen")
    }

    pub(crate) fn issue_command(
        &mut self,
        input: &[u8],
        command: Command,
    ) -> Result<(), io::Error> {
        if !self.command.as_mut().is_some_and(|cmd| cmd.is_running()) {
            self.command = Some(IssuedCommand::spawn(input, command)?);
        }

        Ok(())
    }

    pub(crate) fn issue_subscreen_command(
        &mut self,
        terminal: &mut Terminal,
        command: Command,
    ) -> Result<(), io::Error> {
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

    pub(crate) fn handle_command_output(&mut self) {
        if let Some(cmd) = &mut self.command {
            cmd.read_command_output_to_buffer();

            if cmd.just_finished() {
                self.screen_mut().update();
            }
        }
    }
}

fn main() -> io::Result<()> {
    let mut state = create_initial_state(cli::Cli::parse())?;
    let mut terminal = Terminal::new(CrosstermBackend::new(BufWriter::new(stderr())))?;

    terminal.hide_cursor()?;

    enable_raw_mode()?;
    stderr().execute(EnterAlternateScreen)?;

    run_app(&mut terminal, &mut state)?;

    stderr().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

fn create_initial_state(args: cli::Cli) -> io::Result<State> {
    let screens = match args.command {
        Some(cli::Commands::Show { git_show_args }) => {
            vec![screen::show::create(git_show_args)]
        }
        Some(cli::Commands::Log { git_log_args }) => {
            vec![screen::log::create(git_log_args)]
        }
        Some(cli::Commands::Diff { git_diff_args }) => {
            vec![screen::diff::create(git_diff_args)]
        }
        None => vec![screen::status::create()],
    };

    Ok(State {
        quit: false,
        screens,
        pending_transient_op: TransientOp::None,
        command: None,
    })
}

fn run_app(terminal: &mut Terminal, state: &mut State) -> Result<(), io::Error> {
    while !state.quit {
        if let Some(_screen) = state.screens.last_mut() {
            terminal.draw(|frame| ui::ui(frame, state))?;

            state.handle_command_output();
        }

        handle_events(terminal, state)?;

        if let Some(screen) = state.screens.last_mut() {
            screen.clamp_cursor();
        }
    }

    Ok(())
}

fn handle_events(terminal: &mut Terminal, state: &mut State) -> io::Result<()> {
    if !event::poll(std::time::Duration::from_millis(100))? {
        return Ok(());
    }

    let Some(screen) = state.screens.last_mut() else {
        panic!("No screen");
    };

    match event::read()? {
        Event::Resize(w, h) => screen.size = (w, h),
        Event::Key(key) => {
            if key.kind == KeyEventKind::Press {
                state.clear_finished_command();

                handle_op(terminal, state, key)?;
            }
        }
        _ => (),
    }

    Ok(())
}

fn handle_op(
    terminal: &mut Terminal,
    state: &mut State,
    key: event::KeyEvent,
) -> Result<(), io::Error> {
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
                        screen.update();
                    } else {
                        state.quit = true
                    }
                }
            }
            Refresh => state.screen_mut().update(),
            ToggleSection => state.screen_mut().toggle_section(),
            SelectPrevious => state.screen_mut().select_previous(),
            SelectNext => state.screen_mut().select_next(),
            HalfPageUp => state.screen_mut().scroll_half_page_up(),
            HalfPageDown => state.screen_mut().scroll_half_page_down(),
            Commit => {
                state.issue_subscreen_command(terminal, git::commit_cmd())?;
                state.screen_mut().update();
            }
            CommitAmend => {
                state.issue_subscreen_command(terminal, git::commit_amend_cmd())?;
                state.screen_mut().update();
            }
            Transient(op) => state.pending_transient_op = op,
            LogCurrent => goto_log_screen(&mut state.screens),
            FetchAll => {
                state.issue_command(&[], git::fetch_all_cmd())?;
                state.screen_mut().update();
            }
            PullRemote => state.issue_command(&[], git::pull_cmd())?,
            PushRemote => state.issue_command(&[], git::push_cmd())?,
            Target(target_op) => {
                if let Some(act) = &state.screen_mut().get_selected_item().target_data.clone() {
                    if let Some(mut closure) = closure_by_target_op(act, &target_op) {
                        closure(terminal, state);
                    }
                }
            }
            RebaseAbort => {
                state.issue_command(&[], git::rebase_abort_cmd())?;
                state.screen_mut().update();
            }
            RebaseContinue => {
                state.issue_command(&[], git::rebase_continue_cmd())?;
                state.screen_mut().update();
            }
            ShowRefs => goto_refs_screen(&mut state.screens),
        }
    }

    Ok(())
}

pub(crate) fn list_target_ops<'a>(
    target: &'a TargetData,
) -> impl Iterator<Item = &'static TargetOp> + 'a {
    TargetOp::list_all().filter(|target_op| closure_by_target_op(target, target_op).is_some())
}

type OpClosure<'a> = Box<dyn FnMut(&mut Terminal, &mut State) + 'a>;

pub(crate) fn closure_by_target_op<'a>(
    target: &'a TargetData,
    target_op: &TargetOp,
) -> Option<OpClosure<'a>> {
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
            std::fs::remove_file(f.clone()).expect("Error removing file");
            state.screen_mut().update();
        })),
        (Discard, Delta(d)) => {
            if d.old_file == d.new_file {
                cmd_arg(git::checkout_file_cmd, &d.old_file)
            } else {
                // TODO
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

fn goto_show_screen(r: String) -> Option<Box<dyn FnMut(&mut Terminal, &mut State)>> {
    Some(Box::new(move |_terminal, state| {
        state.screens.push(screen::show::create(vec![r.clone()]));
    }))
}

fn editor(file: String, line: Option<u32>) -> Option<Box<dyn FnMut(&mut Terminal, &mut State)>> {
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

        state.screen_mut().update();
    }))
}

fn cmd(
    input: Vec<u8>,
    command: fn() -> Command,
) -> Option<Box<dyn FnMut(&mut Terminal, &mut State)>> {
    Some(Box::new(move |_terminal, state| {
        state
            .issue_command(&input, command())
            .expect("Error unstaging hunk");
        state.screen_mut().update();
    }))
}

fn cmd_arg(
    command: fn(&str) -> Command,
    arg: &String,
) -> Option<Box<dyn FnMut(&mut Terminal, &mut State)>> {
    let arg_clone = arg.clone();
    Some(Box::new(move |_terminal, state| {
        state
            .issue_command(&[], command(&arg_clone))
            .expect("Error unstaging hunk");
        state.screen_mut().update();
    }))
}

fn subscreen_arg(
    command: fn(&str) -> Command,
    arg: &String,
) -> Option<Box<dyn FnMut(&mut Terminal, &mut State)>> {
    let arg_clone = arg.clone();
    Some(Box::new(move |terminal, state| {
        state
            .issue_subscreen_command(terminal, command(&arg_clone))
            .expect("Error issuing command");
        state.screen_mut().update();
    }))
}

fn goto_log_screen(screens: &mut Vec<Screen>) {
    screens.drain(1..);
    screens.push(screen::log::create(vec![]));
}

fn goto_refs_screen(screens: &mut Vec<Screen>) {
    screens.drain(1..);
    screens.push(screen::show_refs::create());
}
