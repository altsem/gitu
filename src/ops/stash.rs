use super::{Action, OpTrait};
use crate::{
    error::Error,
    items::TargetData,
    menu::arg::Arg,
    state::{PromptParams, State},
    term::Term,
    Res,
};
use git2::{Repository, Status, StatusOptions};
use std::{process::Command, rc::Rc};

pub(crate) fn init_args() -> Vec<Arg> {
    vec![
        Arg::new_flag("--include-untracked", "Also save untracked files", true),
        Arg::new_flag("--all", "Also save untracked and ignored files", false),
    ]
}

pub(crate) struct Stash;
impl OpTrait for Stash {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(move |state: &mut State, term: &mut Term| {
            let message = state.prompt(
                term,
                &PromptParams {
                    prompt: "Stash message",
                    ..Default::default()
                },
            )?;

            stash_push(state, term, &message)?;
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "both".into()
    }
}

fn stash_push(state: &mut State, term: &mut Term, input: &str) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.args(["stash", "push"]);
    cmd.args(state.pending_menu.as_ref().unwrap().args());
    if !input.is_empty() {
        cmd.args(["--message", input]);
    }

    state.close_menu();
    state.run_cmd(term, &[], cmd)?;
    Ok(())
}

pub(crate) struct StashIndex;
impl OpTrait for StashIndex {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(move |state: &mut State, term: &mut Term| {
            let message = state.prompt(
                term,
                &PromptParams {
                    prompt: "Stash message",
                    ..Default::default()
                },
            )?;

            stash_push_index(state, term, &message)?;
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "index".into()
    }
}

fn stash_push_index(state: &mut State, term: &mut Term, input: &str) -> Res<()> {
    let mut cmd = Command::new("git");
    // --all / --unclude-untracked are not allowed together with --staged
    cmd.args(["stash", "push", "--staged"]);
    if !input.is_empty() {
        cmd.args(["--message", input]);
    }

    state.close_menu();
    state.run_cmd(term, &[], cmd)?;
    Ok(())
}

pub(crate) struct StashWorktree;
impl OpTrait for StashWorktree {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|state: &mut State, term: &mut Term| -> Res<()> {
            if is_working_tree_empty(&state.repo)? {
                state.close_menu();
                return Err(Error::StashWorkTreeEmpty);
            }

            let message = state.prompt(
                term,
                &PromptParams {
                    prompt: "Stash message",
                    ..Default::default()
                },
            )?;

            stash_worktree(state, term, &message)?;
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "worktree".into()
    }
}

fn is_working_tree_empty(repo: &Repository) -> Res<bool> {
    let statuses = repo
        .statuses(Some(
            StatusOptions::new()
                .include_untracked(true)
                .include_ignored(false),
        ))
        .map_err(Error::GitStatus)?;

    let is_working_tree_not_empty = statuses.iter().any(|e| {
        e.status().intersects(
            Status::WT_NEW
                | Status::WT_MODIFIED
                | Status::WT_DELETED
                | Status::WT_RENAMED
                | Status::WT_TYPECHANGE,
        )
    });

    Ok(!is_working_tree_not_empty)
}

fn stash_worktree(state: &mut State, term: &mut Term, input: &str) -> Res<()> {
    let need_to_stash_index = is_something_staged(&state.repo)?;

    let mut cmd = Command::new("git");
    cmd.args(["stash", "push"]);
    cmd.args(state.pending_menu.as_ref().unwrap().args());

    state.close_menu();

    if need_to_stash_index {
        let mut cmd = Command::new("git");
        cmd.args(["stash", "push", "--staged"]);
        state.run_cmd(term, &[], cmd)?;
    }

    if !input.is_empty() {
        cmd.args(["--message", input]);
    }
    state.run_cmd(term, &[], cmd)?;

    if need_to_stash_index {
        let mut cmd = Command::new("git");
        cmd.args(["stash", "pop", "-q", "1"]);
        state.run_cmd(term, &[], cmd)?;
    }

    Ok(())
}

fn is_something_staged(repo: &Repository) -> Res<bool> {
    let statuses = repo
        .statuses(Some(
            StatusOptions::new()
                .include_untracked(true)
                .include_ignored(false),
        ))
        .map_err(Error::GitStatus)?;

    Ok(statuses.iter().any(|e| {
        e.status().intersects(
            Status::INDEX_NEW
                | Status::INDEX_MODIFIED
                | Status::INDEX_DELETED
                | Status::INDEX_RENAMED
                | Status::INDEX_TYPECHANGE,
        )
    }))
}

pub(crate) struct StashKeepIndex;
impl OpTrait for StashKeepIndex {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(move |state: &mut State, term: &mut Term| {
            let message = state.prompt(
                term,
                &PromptParams {
                    prompt: "Stash message",
                    ..Default::default()
                },
            )?;

            stash_push_keep_index(state, term, &message)?;
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "keeping index".into()
    }
}

fn stash_push_keep_index(state: &mut State, term: &mut Term, input: &str) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.args(["stash", "push", "--keep-index"]);
    cmd.args(state.pending_menu.as_ref().unwrap().args());
    if !input.is_empty() {
        cmd.args(["--message", input]);
    }

    state.close_menu();
    state.run_cmd(term, &[], cmd)?;
    Ok(())
}

pub(crate) struct StashPop;
impl OpTrait for StashPop {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(move |state: &mut State, term: &mut Term| {
            let input = state.prompt(
                term,
                &PromptParams {
                    prompt: "Pop stash",
                    create_default_value: Box::new(selected_stash),
                    ..Default::default()
                },
            )?;

            stash_pop(state, term, &input)?;
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "pop".into()
    }
}

fn stash_pop(state: &mut State, term: &mut Term, input: &str) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.args(["stash", "pop", "-q"]);
    cmd.arg(input);

    state.close_menu();
    state.run_cmd(term, &[], cmd)?;
    Ok(())
}

pub(crate) struct StashApply;
impl OpTrait for StashApply {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(move |state: &mut State, term: &mut Term| {
            let input = state.prompt(
                term,
                &PromptParams {
                    prompt: "Apply stash",
                    create_default_value: Box::new(selected_stash),
                    ..Default::default()
                },
            )?;

            stash_apply(state, term, &input)?;
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "apply".into()
    }
}

fn stash_apply(state: &mut State, term: &mut Term, input: &str) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.args(["stash", "apply", "-q"]);
    cmd.arg(input);

    state.close_menu();
    state.run_cmd(term, &[], cmd)?;
    Ok(())
}

pub(crate) struct StashDrop;
impl OpTrait for StashDrop {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(move |state: &mut State, term: &mut Term| {
            let input = state.prompt(
                term,
                &PromptParams {
                    prompt: "Drop stash",
                    create_default_value: Box::new(selected_stash),
                    ..Default::default()
                },
            )?;

            stash_drop(state, term, &input)?;
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "drop".into()
    }
}

fn stash_drop(state: &mut State, term: &mut Term, input: &str) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.args(["stash", "drop"]);
    cmd.arg(input);

    state.close_menu();
    state.run_cmd(term, &[], cmd)?;
    Ok(())
}

fn selected_stash(state: &State) -> Option<String> {
    match state.screen().get_selected_item().target_data {
        Some(TargetData::Stash { id, commit: _ }) => Some(id.to_string()),
        _ => Some("0".to_string()),
    }
}
