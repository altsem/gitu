use std::{ffi::OsString, process::Command, rc::Rc};

use crate::{items::TargetData, menu::arg::Arg, state::State, term::Term, Res};
use derive_more::*;

use super::{create_prompt_with_default, selected_rev, Action, OpTrait};

pub(crate) fn get_args() -> Vec<Arg> {
    vec![
        // -m Replay merge relative to parent (--mainline=)
        Arg::new_flag("--edit", "Edit commit message", true),
        // =s Strategy (--strategy=)
        Arg::new_flag("--signoff", "Add Signed-off-by lines", false),
    ]
}

#[derive(Display)]
#[display(fmt = "Revert abort")]
pub(crate) struct RevertAbort;
impl OpTrait for RevertAbort {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|state: &mut State, term: &mut Term| {
            let mut cmd = Command::new("git");
            cmd.args(["revert", "--abort"]);
            state.run_cmd_interactive(term, cmd)?;
            Ok(())
        }))
    }
}

#[derive(Display)]
#[display(fmt = "Revert continue")]
pub(crate) struct RevertContinue;
impl OpTrait for RevertContinue {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|state: &mut State, term: &mut Term| {
            let mut cmd = Command::new("git");
            cmd.args(["revert", "--continue"]);
            state.run_cmd_interactive(term, cmd)?;
            Ok(())
        }))
    }
}

#[derive(Display)]
#[display(fmt = "Revert commit")]
pub(crate) struct RevertCommit;
impl OpTrait for RevertCommit {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(create_prompt_with_default(
            "Revert commit",
            revert_commit,
            selected_rev,
        ))
    }
}

fn revert_commit(state: &mut State, term: &mut Term, args: &[OsString], input: &str) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.args(["revert"]);
    cmd.args(args);
    cmd.arg(input);
    state.run_cmd_interactive(term, cmd)
}
