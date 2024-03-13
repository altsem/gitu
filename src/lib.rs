pub mod cli;
pub mod config;
mod git;
mod git2_opts;
mod items;
mod keybinds;
mod ops;
mod prompt;
mod screen;
pub mod state;
pub mod term;
mod ui;

use crate::ops::SubmenuOp;
use crossterm::event::{self};
use git2::Repository;
use items::Item;
use itertools::Itertools;
use ops::{Action, Op, TargetOp};
use state::State;
use std::{borrow::Cow, error::Error, iter, path::PathBuf, process::Command};
use term::Term;

const APP_NAME: &str = "gitu";

pub type Res<T> = Result<T, Box<dyn Error>>;

pub(crate) struct CmdMetaBuffer {
    pub(crate) args: Cow<'static, str>,
    pub(crate) out: Option<String>,
}

pub(crate) struct ErrorBuffer(String);

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

// TODO Split remaining parts into modules at crate::ops
pub(crate) fn handle_op(state: &mut State, op: Op, term: &mut Term) -> Res<()> {
    let was_submenu = state.pending_submenu_op != SubmenuOp::None;
    state.pending_submenu_op = SubmenuOp::None;

    match op {
        Op::Quit => state.handle_quit(was_submenu)?,
        Op::Refresh => state.screen_mut().update()?,

        Op::Submenu(op) => state.pending_submenu_op = op,

        Op::Target(TargetOp::Discard) => ops::OpTrait::trigger(&op, state, term)?,
        Op::Target(target_op) => {
            if let Some(mut action) = ops::get_action(state.clone_target_data(), target_op) {
                action(state, term)?;
            }
        }
        _ => ops::OpTrait::trigger(&op, state, term)?,
    }

    Ok(())
}
