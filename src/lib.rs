pub mod cli;
pub mod config;
mod git;
mod git2_opts;
mod items;
mod keybinds;
mod menu;
mod ops;
mod prompt;
mod screen;
pub mod state;
pub mod term;
#[cfg(test)]
mod tests;
mod ui;

use crossterm::event::{self};
use git2::Repository;
use items::Item;
use itertools::Itertools;
use ops::{Action, Op};
use state::State;
use std::{borrow::Cow, error::Error, iter, path::PathBuf, process::Command, rc::Rc};
use term::Term;

pub type Res<T> = Result<T, Box<dyn Error>>;

pub(crate) enum CmdLogEntry {
    Cmd {
        args: Cow<'static, str>,
        out: Option<Cow<'static, str>>,
    },
    Error(String),
}

fn command_args(cmd: &Command) -> Cow<'static, str> {
    iter::once(cmd.get_program().to_string_lossy())
        .chain(cmd.get_args().map(|arg| arg.to_string_lossy()))
        .join(" ")
        .into()
}

pub fn run(args: &cli::Args, term: &mut Term) -> Res<()> {
    log::debug!("Finding git dir");
    let dir = PathBuf::from(
        String::from_utf8(
            Command::new("git")
                .args(["rev-parse", "--show-toplevel"])
                .output()?
                .stdout,
        )?
        .trim_end(),
    );

    log::debug!("Opening repo");
    let repo = Repository::open_from_env()?;
    repo.set_workdir(&dir, false)?;

    log::debug!("Initializing config");
    let config = config::init_config()?;

    log::debug!("Creating initial state");
    let mut state = state::State::create(repo, term.size()?, args, config)?;

    log::debug!("Drawing initial frame");
    term.draw(|frame| ui::ui(frame, &mut state))?;

    if args.print {
        return Ok(());
    }

    while !state.quit {
        log::debug!("Awaiting event");
        let event = event::read()?;

        log::debug!("Updating");
        state.update(term, &[event])?;
    }

    Ok(())
}

pub(crate) fn handle_op(state: &mut State, op: Op, term: &mut Term) -> Res<()> {
    let target = state.screen().get_selected_item().target_data.as_ref();
    if let Some(mut action) = op.implementation().get_action(target) {
        let result = Rc::get_mut(&mut action).unwrap()(state, term);

        close_menu(state, op);

        if let Err(error) = result {
            state
                .current_cmd_log_entries
                .push(CmdLogEntry::Error(error.to_string()));
        }
    }

    Ok(())
}

fn close_menu(state: &mut State, op: Op) {
    match op {
        Op::Menu(_) => (),
        Op::ToggleArg(_) => (),
        _ => state.pending_menu = None,
    }
}
