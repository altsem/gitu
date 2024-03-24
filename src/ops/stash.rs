use super::{Action, OpTrait};
use crate::{items::TargetData, prompt::PromptData, state::State, term::Term, Res};
use derive_more::Display;
use std::{process::Command, rc::Rc};
use tui_prompts::State as _;

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
                // TODO: How to show all 3 commands? We show only the last one in the current
                // implementation.

                // 1. Stash index (stash@0: index, ...)
                let mut cmd = Command::new("git");
                cmd.args(["stash", "push", "--staged"]);
                state.run_external_cmd(term, &[], cmd)?;

                // 2. Stash everything else (stash@0: worktree, stash@1: index, ...)
                let mut cmd = Command::new("git");
                cmd.args(["stash", "push", "--include-untracked"]);
                if !input.is_empty() {
                    cmd.args(["--message", &input]);
                }
                state.run_external_cmd(term, &[], cmd)?;

                // 3. Pop stash with index (at stash@1)
                let mut cmd = Command::new("git");
                cmd.args(["stash", "pop", "1"]);
                state.run_external_cmd(term, &[], cmd)?;

                state.prompt.reset(term)?;
            }
            Ok(())
        };

        Some(Rc::new(
            move |state: &mut State, _term: &mut Term| -> Res<()> {
                state.prompt.set(PromptData {
                    prompt_text: "Name of the stash:".into(),
                    update_fn: Rc::new(update_fn),
                });
                Ok(())
            },
        ))
    }
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

            let mut cmd = Command::new("git");
            cmd.args(["stash", "push"]);
            cmd.args(args);
            if !input.is_empty() {
                cmd.args(["--message".into(), input]);
            }

            state.run_external_cmd(term, &[], cmd)?;
            state.prompt.reset(term)?;
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
    Rc::new(|state: &mut State, _term: &mut Term| -> Res<()> {
        let prompt_text = if let Some(id) = default_target_stash(state) {
            format!("Stash index (default {}):", id).into()
        } else {
            "Stash index (default 0):".into()
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
            let stash_id: usize = match (input.parse(), default_target_stash(state)) {
                (Err(_), None) => 0,
                (Err(_), Some(default)) => default,
                (Ok(value), _) => value,
            };

            let mut cmd = Command::new("git");
            cmd.args(["stash", command, stash_id.to_string().as_str()]);

            state.run_external_cmd(term, &[], cmd)?;
            state.prompt.reset(term)?;
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
