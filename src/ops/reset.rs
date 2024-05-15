use super::{create_prompt_with_default, selected_rev, OpTrait};
use crate::{items::TargetData, menu::arg::Arg, state::State, term::Term, Action, Res};
use derive_more::Display;
use std::{ffi::OsString, process::Command};

pub(crate) fn init_args() -> Vec<Arg> {
    vec![]
}

#[derive(Display)]
#[display(fmt = "Reset soft")]
pub(crate) struct ResetSoft;
impl OpTrait for ResetSoft {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(create_prompt_with_default(
            "Soft reset to",
            reset_soft,
            selected_rev,
        ))
    }
}

fn reset_soft(state: &mut State, term: &mut Term, args: &[OsString], input: &str) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.args(["reset", "--soft"]);
    cmd.args(args);
    cmd.arg(input);
    state.run_cmd(term, &[], cmd)
}

#[derive(Display)]
#[display(fmt = "Reset mixed")]
pub(crate) struct ResetMixed;
impl OpTrait for ResetMixed {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(create_prompt_with_default(
            "Mixed reset to",
            reset_mixed,
            selected_rev,
        ))
    }
}

fn reset_mixed(state: &mut State, term: &mut Term, args: &[OsString], input: &str) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.args(["reset", "--mixed"]);
    cmd.args(args);
    cmd.arg(input);
    state.run_cmd(term, &[], cmd)
}

#[derive(Display)]
#[display(fmt = "Reset hard")]
pub(crate) struct ResetHard;
impl OpTrait for ResetHard {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(create_prompt_with_default(
            "Hard reset to",
            reset_hard,
            selected_rev,
        ))
    }
}

fn reset_hard(state: &mut State, term: &mut Term, args: &[OsString], input: &str) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.args(["reset", "--hard"]);
    cmd.args(args);
    cmd.arg(input);
    state.run_cmd(term, &[], cmd)
}
