use super::{subscreen_arg, Action, OpTrait};
use crate::{items::TargetData, menu::arg::Arg, state::State, term::Term};
use derive_more::Display;
use std::{ffi::OsStr, process::Command, rc::Rc};

pub(crate) const ARGS: &[Arg] = &[];

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Commit")]
pub(crate) struct Commit;
impl OpTrait for Commit {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|state: &mut State, term: &mut Term| {
            let mut cmd = Command::new("git");
            cmd.args(["commit"]);

            state.run_cmd_interactive(term, cmd)?;
            Ok(())
        }))
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Commit amend")]
pub(crate) struct CommitAmend;
impl OpTrait for CommitAmend {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|state: &mut State, term: &mut Term| {
            let mut cmd = Command::new("git");
            cmd.args(["commit", "--amend"]);

            state.run_cmd_interactive(term, cmd)?;
            Ok(())
        }))
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Commit fixup")]
pub(crate) struct CommitFixup;
impl OpTrait for CommitFixup {
    fn get_action(&self, target: Option<&TargetData>) -> Option<Action> {
        let action = match target {
            Some(TargetData::Commit(r)) => subscreen_arg(commit_fixup_cmd, r.into()),
            _ => return None,
        };

        Some(action)
    }
    fn is_target_op(&self) -> bool {
        true
    }
}

fn commit_fixup_cmd(reference: &OsStr) -> Command {
    let mut cmd = Command::new("git");
    cmd.args(["commit", "--fixup"]);
    cmd.arg(reference);
    cmd
}
