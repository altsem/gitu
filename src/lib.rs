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

use crate::keybinds::{Op, SubmenuOp};
use crossterm::event::{self};
use git2::Repository;
use items::{Item, TargetData};
use itertools::Itertools;
use keybinds::TargetOp;
use ratatui::prelude::*;
use state::State;
use std::{
    borrow::Cow,
    error::Error,
    ffi::{OsStr, OsString},
    iter,
    path::{Path, PathBuf},
    process::Command,
    rc::Rc,
};
use strum::IntoEnumIterator;

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

pub fn run<B: Backend>(args: &cli::Args, term: &mut Terminal<B>) -> Res<()> {
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
    term.draw(|frame| ui::ui::<B>(frame, &mut state))?;

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

fn get_action<B: Backend>(
    target_data: Option<TargetData>,
    target_op: TargetOp,
) -> Option<Action<B>> {
    target_data.and_then(|data| action_by_target_op::<B>(data, &target_op))
}

pub(crate) fn list_target_ops<B: Backend>(
    data: &TargetData,
) -> impl Iterator<Item = (TargetOp, TargetData)> + '_ {
    TargetOp::iter()
        .filter(|target_op| action_by_target_op::<B>(data.clone(), target_op).is_some())
        .map(|op| (op, data.clone()))
}

pub(crate) fn handle_op<B: Backend>(state: &mut State, op: Op, term: &mut Terminal<B>) -> Res<()> {
    use Op::*;

    let was_submenu = state.pending_submenu_op != SubmenuOp::None;
    state.pending_submenu_op = SubmenuOp::None;

    // TODO Move into separate modules
    match op {
        Quit => state.handle_quit(was_submenu)?,
        Refresh => state.screen_mut().update()?,
        ToggleSection => state.screen_mut().toggle_section(),
        SelectPrevious => state.screen_mut().select_previous(),
        SelectNext => state.screen_mut().select_next(),
        HalfPageUp => state.screen_mut().scroll_half_page_up(),
        HalfPageDown => state.screen_mut().scroll_half_page_down(),

        CheckoutNewBranch => ops::OpTrait::<B>::trigger(&op, state, term)?,
        Commit => ops::OpTrait::<B>::trigger(&op, state, term)?,
        CommitAmend => ops::OpTrait::<B>::trigger(&op, state, term)?,
        Submenu(op) => state.pending_submenu_op = op,
        LogCurrent => state.goto_log_screen(None),
        FetchAll => state.run_external_cmd(term, &[], git::fetch_all_cmd())?,
        Pull => state.run_external_cmd(term, &[], git::pull_cmd())?,
        Push => state.run_external_cmd(term, &[], git::push_cmd())?,
        Target(TargetOp::Discard) => ops::OpTrait::<B>::trigger(&op, state, term)?,
        Target(target_op) => state.try_dispatch_target_action(target_op, term)?,
        RebaseAbort => state.run_external_cmd(term, &[], git::rebase_abort_cmd())?,
        RebaseContinue => state.issue_subscreen_command(term, git::rebase_continue_cmd())?,
        ShowRefs => state.goto_refs_screen(),
    }

    Ok(())
}

type Action<B> = Box<dyn FnMut(&mut state::State, &mut Terminal<B>) -> Res<()>>;

/// Retrieves the 'implementation' of a `TargetOp`.
/// These are `Option<OpClosure>`s, so that the mappings
/// can be introspected.
pub(crate) fn action_by_target_op<B: Backend>(
    target: TargetData,
    target_op: &TargetOp,
) -> Option<Action<B>> {
    use TargetData::*;
    use TargetOp::*;

    match (target_op, target) {
        (Show, Commit(r) | Branch(r)) => goto_show_screen(r.clone()),
        (Show, File(u)) => editor(u.as_path(), None),
        (Show, Delta(d)) => editor(d.new_file.as_path(), None),
        (Show, Hunk(h)) => editor(h.new_file.as_path(), Some(h.first_diff_line())),
        (Stage, File(u)) => cmd_arg(git::stage_file_cmd, u.into()),
        (Stage, Delta(d)) => cmd_arg(git::stage_file_cmd, d.new_file.into()),
        (Stage, Hunk(h)) => cmd(h.format_patch().into_bytes(), git::stage_patch_cmd),
        (Unstage, Delta(d)) => cmd_arg(git::unstage_file_cmd, d.new_file.into()),
        (Unstage, Hunk(h)) => cmd(h.format_patch().into_bytes(), git::unstage_patch_cmd),
        (RebaseInteractive, Commit(r) | Branch(r)) => {
            subscreen_arg(git::rebase_interactive_cmd, r.into())
        }
        (CommitFixup, Commit(r)) => subscreen_arg(git::commit_fixup_cmd, r.into()),
        (RebaseAutosquash, Commit(r) | Branch(r)) => {
            subscreen_arg(git::rebase_autosquash_cmd, r.into())
        }
        (ResetSoft, Commit(r) | Branch(r)) => cmd_arg(git::reset_soft_cmd, r.into()),
        (ResetMixed, Commit(r) | Branch(r)) => cmd_arg(git::reset_mixed_cmd, r.into()),
        (ResetHard, Commit(r) | Branch(r)) => cmd_arg(git::reset_hard_cmd, r.into()),
        (Discard, Branch(r)) => cmd_arg(git::discard_branch, r.into()),
        (Discard, File(f)) => Some(Box::new(move |state, _term| {
            let path = PathBuf::from_iter([
                state.repo.workdir().expect("No workdir").to_path_buf(),
                f.clone(),
            ]);
            std::fs::remove_file(path)?;
            state.screen_mut().update()
        })),
        (Discard, Delta(d)) => {
            if d.old_file == d.new_file {
                cmd_arg(git::checkout_file_cmd, d.old_file.into())
            } else {
                // TODO Discard file move
                None
            }
        }
        (Discard, Hunk(h)) => cmd(
            h.format_patch().into_bytes(),
            git::discard_unstaged_patch_cmd,
        ),
        (Checkout, Commit(r) | Branch(r)) => cmd_arg(git::checkout_ref_cmd, r.into()),
        (LogOther, Commit(r) | Branch(r)) => Some(Box::new(move |state, _term| {
            state.goto_log_screen(Some(r.clone()));
            Ok(())
        })),
        (_, _) => None,
    }
}

fn goto_show_screen<B: Backend>(r: String) -> Option<Action<B>> {
    Some(Box::new(move |state, term| {
        state.screens.push(
            screen::show::create(
                Rc::clone(&state.config),
                Rc::clone(&state.repo),
                term.size()?,
                r.clone(),
            )
            .expect("Couldn't create screen"),
        );
        Ok(())
    }))
}

fn editor<B: Backend>(file: &Path, line: Option<u32>) -> Option<Action<B>> {
    let file = file.to_str().unwrap().to_string();

    Some(Box::new(move |state, term| {
        const EDITOR_VARS: [&str; 3] = ["GIT_EDITOR", "VISUAL", "EDITOR"];
        let configured_editor = EDITOR_VARS
            .into_iter()
            .find_map(|var| std::env::var(var).ok());

        let Some(editor) = configured_editor else {
            return Err(format!(
                "No editor environment variable set ({})",
                EDITOR_VARS.join(", ")
            )
            .into());
        };

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
            .issue_subscreen_command(term, cmd)
            .map_err(|err| format!("Couldn't open editor {} due to: {}", editor, err))?;

        state.screen_mut().update()
    }))
}

fn cmd<B: Backend>(input: Vec<u8>, command: fn() -> Command) -> Option<Action<B>> {
    Some(Box::new(move |state, term| {
        state.run_external_cmd(term, &input, command())
    }))
}

fn cmd_arg<B: Backend>(command: fn(&OsStr) -> Command, arg: OsString) -> Option<Action<B>> {
    Some(Box::new(move |state, term| {
        state.run_external_cmd(term, &[], command(&arg))
    }))
}

fn subscreen_arg<B: Backend>(command: fn(&OsStr) -> Command, arg: OsString) -> Option<Action<B>> {
    Some(Box::new(move |state, term| {
        state.issue_subscreen_command(term, command(&arg))
    }))
}
