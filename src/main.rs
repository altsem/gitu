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

use clap::Parser;
use crossterm::{
    event::{self, Event, KeyEventKind},
    terminal::{
        self, disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    },
    ExecutableCommand,
};
use diff::Hunk;
use items::{Item, TargetData};
use keybinds::{Op, TargetOp};
use ratatui::prelude::CrosstermBackend;
use screen::Screen;
use std::{
    io::{self, stderr, Stderr},
    process::{Command, Stdio},
};

type Terminal = ratatui::Terminal<CrosstermBackend<Stderr>>;

lazy_static::lazy_static! {
    static ref USE_DELTA: bool = Command::new("delta").output().map(|out| out.status.success()).unwrap_or(false);
}

struct State {
    quit: bool,
    screens: Vec<Screen>,
    terminal: Terminal,
}

impl State {
    fn screen_mut(&mut self) -> &mut Screen {
        self.screens.last_mut().expect("No screen")
    }
}

fn main() -> io::Result<()> {
    let mut state = create_initial_state(cli::Cli::parse())?;

    state.terminal.hide_cursor()?;

    enable_raw_mode()?;
    stderr().execute(EnterAlternateScreen)?;

    run_app(&mut state)?;

    stderr().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

fn create_initial_state(args: cli::Cli) -> io::Result<State> {
    let size = terminal::size()?;
    let screens = match args.command {
        Some(cli::Commands::Show { git_show_args }) => {
            vec![screen::show::create(size, git_show_args)]
        }
        Some(cli::Commands::Log { git_log_args }) => vec![screen::log::create(size, git_log_args)],
        None => vec![screen::status::create(size)],
    };

    Ok(State {
        quit: false,
        screens,
        terminal: Terminal::new(CrosstermBackend::new(stderr()))?,
    })
}

fn run_app(state: &mut State) -> Result<(), io::Error> {
    while !state.quit {
        if let Some(screen) = state.screens.last_mut() {
            state.terminal.draw(|frame| ui::ui(frame, screen))?;
            screen.handle_command_output();
        }

        handle_events(state)?;

        if let Some(screen) = state.screens.last_mut() {
            screen.clamp_cursor();
        }
    }

    Ok(())
}

fn handle_events(state: &mut State) -> io::Result<()> {
    // TODO Won't need to poll all the time if command outputs were handled async
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
                screen.clear_finished_command();

                handle_op(state, key)?;
            }
        }
        _ => (),
    }

    if state.quit {
        state.screens.pop();
        if let Some(screen) = state.screens.last_mut() {
            state.quit = false;
            screen.refresh_items();
        }
    }

    Ok(())
}

fn handle_op(state: &mut State, key: event::KeyEvent) -> Result<(), io::Error> {
    let Some(screen) = state.screens.last_mut() else {
        panic!("No screen");
    };

    if let Some(op) = keybinds::op_of_key_event(key) {
        match op {
            Op::Quit => state.quit = true,
            Op::Refresh => screen.refresh_items(),
            Op::ToggleSection => screen.toggle_section(),
            Op::SelectPrevious => screen.select_previous(),
            Op::SelectNext => screen.select_next(),
            Op::HalfPageUp => screen.scroll_half_page_up(),
            Op::HalfPageDown => screen.scroll_half_page_down(),
            Op::Log => goto_log_screen(&mut state.screens),
            Op::Fetch => {
                screen.issue_command(&[], git::fetch_all_cmd())?;
                screen.refresh_items();
            }
            Op::Commit => {
                open_subscreen(&mut state.terminal, &[], git::commit_cmd())?;
                screen.refresh_items();
            }
            Op::Push => screen.issue_command(&[], git::push_cmd())?,
            Op::Pull => screen.issue_command(&[], git::pull_cmd())?,
            Op::Target(target_op) => {
                if let Some(act) = &screen.get_selected_item().target_data.clone() {
                    if let Some(mut function) = function_by_target_op(act, target_op) {
                        function(state);
                    }
                }
            }
        }
    }

    Ok(())
}

pub(crate) fn list_target_ops(target: &TargetData) -> Vec<TargetOp> {
    TargetOp::list_all()
        .filter_map(|target_op| {
            if function_by_target_op(target, target_op).is_some() {
                Some(target_op)
            } else {
                None
            }
        })
        .collect()
}

pub(crate) fn function_by_target_op(
    target: &TargetData,
    target_op: TargetOp,
) -> Option<Box<dyn FnMut(&mut State)>> {
    match (target, target_op) {
        (TargetData::Ref(r), TargetOp::Show) => {
            let reference = r.clone();
            Some(Box::new(move |state| {
                goto_show_screen(&reference, &mut state.screens);
            }))
        }
        (TargetData::Ref(_), TargetOp::Stage) => None,
        (TargetData::Ref(_), TargetOp::Unstage) => None,
        (TargetData::Untracked(u), TargetOp::Show) => {
            let untracked = u.clone();
            Some(Box::new(move |state| {
                open_subscreen(&mut state.terminal, &[], editor_cmd(&untracked, None))
                    .expect("Error opening editor");
                state.screen_mut().refresh_items();
            }))
        }
        (TargetData::Untracked(u), TargetOp::Stage) => {
            let untracked = u.clone();
            Some(Box::new(move |state| {
                state
                    .screen_mut()
                    .issue_command(&[], git::stage_file_cmd(&untracked))
                    .expect("Error staging file");
                state.screen_mut().refresh_items();
            }))
        }
        (TargetData::Untracked(_), TargetOp::Unstage) => None,
        (TargetData::Delta(d), TargetOp::Show) => {
            let delta = d.clone();
            Some(Box::new(move |state| {
                let terminal: &mut Terminal = &mut state.terminal;
                open_subscreen(terminal, &[], editor_cmd(&delta.new_file, None))
                    .expect("Error opening editor");
                state.screen_mut().refresh_items();
            }))
        }
        (TargetData::Delta(d), TargetOp::Stage) => {
            let delta = d.clone();
            Some(Box::new(move |state| {
                state
                    .screen_mut()
                    .issue_command(&[], git::stage_file_cmd(&delta.new_file))
                    .expect("Error staging file");
                state.screen_mut().refresh_items();
            }))
        }
        (TargetData::Delta(d), TargetOp::Unstage) => {
            let delta = d.clone();
            Some(Box::new(move |state| {
                state
                    .screen_mut()
                    .issue_command(&[], git::unstage_file_cmd(&delta))
                    .expect("Error unstaging file");
                state.screen_mut().refresh_items();
            }))
        }
        (TargetData::Hunk(h), TargetOp::Show) => {
            let hunk = h.clone();
            Some(Box::new(move |state| {
                open_subscreen(
                    &mut state.terminal,
                    &[],
                    editor_cmd(&hunk.new_file, Some(&hunk)),
                )
                .expect("Error opening editor");
                state.screen_mut().refresh_items();
            }))
        }
        (TargetData::Hunk(h), TargetOp::Stage) => {
            let hunk = h.clone();
            Some(Box::new(move |state| {
                state
                    .screen_mut()
                    .issue_command(hunk.format_patch().as_bytes(), git::stage_patch_cmd())
                    .expect("Error staging hunk");
                state.screen_mut().refresh_items();
            }))
        }
        (TargetData::Hunk(h), TargetOp::Unstage) => {
            let hunk = h.clone();
            Some(Box::new(move |state| {
                state
                    .screen_mut()
                    .issue_command(hunk.format_patch().as_bytes(), git::unstage_patch_cmd())
                    .expect("Error unstaging hunk");
                state.screen_mut().refresh_items();
            }))
        }
    }
}

fn goto_show_screen(reference: &str, screens: &mut Vec<Screen>) {
    let size = terminal::size().expect("Error reading terminal size");
    screens.push(screen::show::create(size, vec![reference.to_string()]));
}

fn goto_log_screen(screens: &mut Vec<Screen>) {
    let size = terminal::size().expect("Error reading terminal size");
    screens.drain(1..);
    screens.push(screen::log::create(size, vec![]));
}

fn editor_cmd(delta: &str, maybe_hunk: Option<&Hunk>) -> Command {
    let editor = std::env::var("EDITOR").expect("EDITOR not set");
    let mut cmd = Command::new(editor.clone());
    let args = match maybe_hunk {
        Some(hunk) => match editor.as_str() {
            "vi" | "vim" | "nvim" | "nano" => {
                vec![format!("+{}", hunk.new_start), delta.to_string()]
            }
            _ => vec![format!("{}:{}", delta, hunk.new_start)],
        },
        None => vec![delta.to_string()],
    };

    cmd.args(args);
    cmd
}

pub(crate) fn open_subscreen(
    terminal: &mut Terminal,
    input: &[u8],
    mut cmd: Command,
) -> Result<(), io::Error> {
    cmd.stdin(Stdio::piped());
    let mut cmd = cmd.spawn()?;

    use std::io::Write;
    cmd.stdin
        .take()
        .expect("Error taking stdin")
        .write_all(input)?;

    cmd.wait()?;

    terminal.hide_cursor()?;
    terminal.clear()?;

    Ok(())
}
