use super::{Action, OpTrait};
use crate::{items::TargetData, menu::arg::Arg, prompt::PromptData, state::State, term::Term, Res};
use derive_more::Display;
use git2::{Repository, Status, StatusOptions};
use std::{process::Command, rc::Rc};
use tui_prompts::State as _;

pub(crate) const ARGS: &[Arg] = &[];

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Stash (include untracked)")]
pub(crate) struct Stash;
impl OpTrait for Stash {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(stash_push_action(["--include-untracked"]))
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Stash index")]
pub(crate) struct StashIndex;
impl OpTrait for StashIndex {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(stash_push_action(["--staged"]))
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Stash working tree")]
pub(crate) struct StashWorktree;
impl OpTrait for StashWorktree {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        let update_fn = move |state: &mut State, term: &mut Term| -> Res<()> {
            if state.prompt.state.status().is_done() {
                let input = state.prompt.state.value().to_string();
                state.prompt.reset(term)?;

                let need_to_stash_index = is_something_staged(&state.repo)?;

                if need_to_stash_index {
                    let mut cmd = Command::new("git");
                    cmd.args(["stash", "push", "--staged"]);
                    state.run_cmd(term, &[], cmd)?;
                }

                let mut cmd = Command::new("git");
                cmd.args(["stash", "push", "--include-untracked"]);
                if !input.is_empty() {
                    cmd.args(["--message", &input]);
                }
                state.run_cmd(term, &[], cmd)?;

                if need_to_stash_index {
                    let mut cmd = Command::new("git");
                    cmd.args(["stash", "pop", "1"]);
                    state.run_cmd(term, &[], cmd)?;
                }
            }
            Ok(())
        };

        Some(Rc::new(
            move |state: &mut State, _term: &mut Term| -> Res<()> {
                if is_working_tree_empty(&state.repo)? {
                    return Err("Cannot stash: working tree is empty".into());
                }
                state.prompt.set(PromptData {
                    prompt_text: "Name of the stash:".into(),
                    update_fn: Rc::new(update_fn),
                });
                Ok(())
            },
        ))
    }
}

fn is_working_tree_empty(repo: &Repository) -> Res<bool> {
    let statuses = repo.statuses(Some(
        StatusOptions::new()
            .include_untracked(true)
            .include_ignored(false),
    ))?;

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

fn is_something_staged(repo: &Repository) -> Res<bool> {
    let statuses = repo.statuses(Some(
        StatusOptions::new()
            .include_untracked(true)
            .include_ignored(false),
    ))?;

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

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Stash keeping index")]
pub(crate) struct StashKeepIndex;
impl OpTrait for StashKeepIndex {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(stash_push_action(["--keep-index", "--include-untracked"]))
    }
}

fn stash_push_action<const N: usize>(args: [&'static str; N]) -> Action {
    Rc::new(move |state: &mut State, _term: &mut Term| -> Res<()> {
        state.prompt.set(PromptData {
            prompt_text: "Name of the stash:".into(),
            update_fn: Rc::new(stash_push_action_prompt_update(args)),
        });
        Ok(())
    })
}

fn stash_push_action_prompt_update<const N: usize>(
    args: [&'static str; N],
) -> impl FnMut(&mut State, &mut Term) -> Res<()> + 'static {
    move |state: &mut State, term: &mut Term| -> Res<()> {
        if state.prompt.state.status().is_done() {
            let input = state.prompt.state.value().to_string();
            state.prompt.reset(term)?;

            let mut cmd = Command::new("git");
            cmd.args(["stash", "push"]);
            cmd.args(args);
            if !input.is_empty() {
                cmd.args(["--message".into(), input]);
            }

            state.run_cmd(term, &[], cmd)?;
        }
        Ok(())
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Pop stash")]
pub(crate) struct StashPop;
impl OpTrait for StashPop {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(stash_target_action("pop"))
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Apply stash")]
pub(crate) struct StashApply;
impl OpTrait for StashApply {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(stash_target_action("apply"))
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Drop stash")]
pub(crate) struct StashDrop;
impl OpTrait for StashDrop {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(stash_target_action("drop"))
    }
}

fn stash_target_action(command: &'static str) -> Action {
    Rc::new(move |state: &mut State, _term: &mut Term| -> Res<()> {
        let prompt_text = if let Some(id) = default_target_stash(state) {
            format!("Stash {} (default {}):", command, id).into()
        } else {
            format!("Stash {} (default 0):", command).into()
        };

        state.prompt.set(PromptData {
            prompt_text,
            update_fn: Rc::new(stash_target_action_prompt_update(command)),
        });
        Ok(())
    })
}

fn stash_target_action_prompt_update(
    command: &str,
) -> impl FnMut(&mut State, &mut Term) -> Res<()> + '_ {
    |state: &mut State, term: &mut Term| -> Res<()> {
        if state.prompt.state.status().is_done() {
            let input = state.prompt.state.value().to_string();
            state.prompt.reset(term)?;

            let stash_id: usize = match (input.parse(), default_target_stash(state)) {
                (Err(_), None) => 0,
                (Err(_), Some(default)) => default,
                (Ok(value), _) => value,
            };

            let mut cmd = Command::new("git");
            cmd.args(["stash", command, stash_id.to_string().as_str()]);

            state.run_cmd(term, &[], cmd)?;
        }
        Ok(())
    }
}

fn default_target_stash(state: &State) -> Option<usize> {
    match state.screen().get_selected_item().target_data {
        Some(TargetData::Stash { id, commit: _ }) => Some(id),
        _ => None,
    }
}
