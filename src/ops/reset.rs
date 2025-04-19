use super::{create_prompt_with_default, selected_rev, OpTrait};
use crate::{items::TargetData, menu::arg::Arg, state::State, Action, Res};
use std::process::Command;

pub(crate) fn init_args() -> Vec<Arg> {
    vec![]
}

pub(crate) struct ResetSoft;
impl OpTrait for ResetSoft {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(create_prompt_with_default(
            "Soft reset to",
            reset_soft,
            selected_rev,
            true,
        ))
    }

    fn display(&self, _state: &State) -> String {
        "soft".into()
    }
}

fn reset_soft(state: &mut State, input: &str) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.args(["reset", "--soft"]);
    cmd.args(state.pending_menu.as_ref().unwrap().args());
    cmd.arg(input);

    state.close_menu();
    state.run_cmd(&[], cmd)
}

pub(crate) struct ResetMixed;
impl OpTrait for ResetMixed {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(create_prompt_with_default(
            "Mixed reset to",
            reset_mixed,
            selected_rev,
            true,
        ))
    }

    fn display(&self, _state: &State) -> String {
        "mixed".into()
    }
}

fn reset_mixed(state: &mut State, input: &str) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.args(["reset", "--mixed"]);
    cmd.args(state.pending_menu.as_ref().unwrap().args());
    cmd.arg(input);

    state.close_menu();
    state.run_cmd(&[], cmd)
}

pub(crate) struct ResetHard;
impl OpTrait for ResetHard {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(create_prompt_with_default(
            "Hard reset to",
            reset_hard,
            selected_rev,
            true,
        ))
    }

    fn display(&self, _state: &State) -> String {
        "hard".into()
    }
}

fn reset_hard(state: &mut State, input: &str) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.args(["reset", "--hard"]);
    cmd.args(state.pending_menu.as_ref().unwrap().args());
    cmd.arg(input);

    state.close_menu();
    state.run_cmd(&[], cmd)
}
