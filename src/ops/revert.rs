use std::{process::Command, rc::Rc};

use crate::{items::TargetData, menu::arg::Arg, state::State, term::Term, Res};

use super::{create_prompt_with_default, selected_rev, Action, OpTrait};

pub(crate) fn init_args() -> Vec<Arg> {
    vec![
        // -m Replay merge relative to parent (--mainline=)
        Arg::new_flag("--edit", "Edit commit message", true),
        // =s Strategy (--strategy=)
        Arg::new_flag("--signoff", "Add Signed-off-by lines", false),
    ]
}

pub(crate) struct RevertAbort;
impl OpTrait for RevertAbort {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|state: &mut State, term: &mut Term| {
            let mut cmd = Command::new("git");
            cmd.args(["revert", "--abort"]);

            state.close_menu();
            state.run_cmd_interactive(term, cmd)?;
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "Abort".into()
    }
}

pub(crate) struct RevertContinue;
impl OpTrait for RevertContinue {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|state: &mut State, term: &mut Term| {
            let mut cmd = Command::new("git");
            cmd.args(["revert", "--continue"]);

            state.close_menu();
            state.run_cmd_interactive(term, cmd)?;
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "Continue".into()
    }
}

pub(crate) struct RevertCommit;
impl OpTrait for RevertCommit {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(create_prompt_with_default(
            "Revert commit",
            revert_commit,
            selected_rev,
            true,
        ))
    }

    fn display(&self, _state: &State) -> String {
        "Revert commit(s)".into()
    }
}

fn revert_commit(state: &mut State, term: &mut Term, input: &str) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.args(["revert"]);
    cmd.args(state.pending_menu.as_ref().unwrap().args());
    cmd.arg(input);

    state.close_menu();
    state.run_cmd_interactive(term, cmd)
}
