use super::{create_rev_prompt, subscreen_arg, Action, OpTrait};
use crate::{items::TargetData, state::State, term::Term, Res};
use derive_more::Display;
use std::{
    ffi::{OsStr, OsString},
    process::Command,
    rc::Rc,
};

pub(crate) fn args() -> &'static [(&'static str, bool)] {
    &[]
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Rebase continue")]
pub(crate) struct RebaseContinue;
impl OpTrait for RebaseContinue {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|state: &mut State, term: &mut Term| {
            let mut cmd = Command::new("git");
            cmd.args(["rebase", "--continue"]);

            state.run_cmd_interactive(term, cmd)?;
            Ok(())
        }))
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Rebase abort")]
pub(crate) struct RebaseAbort;
impl OpTrait for RebaseAbort {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|state: &mut State, term: &mut Term| {
            let mut cmd = Command::new("git");
            cmd.args(["rebase", "--abort"]);

            state.run_cmd(term, &[], cmd)?;
            Ok(())
        }))
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Rebase elsewhere")]
pub(crate) struct RebaseElsewhere;
impl OpTrait for RebaseElsewhere {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(create_rev_prompt("Rebase onto", rebase_elsewhere))
    }
}

fn rebase_elsewhere(state: &mut State, term: &mut Term, result: &str) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.args(["rebase"]);
    cmd.arg(result);

    state.run_cmd_interactive(term, cmd)?;
    Ok(())
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Rebase interactive")]
pub(crate) struct RebaseInteractive;
impl OpTrait for RebaseInteractive {
    fn get_action(&self, target: Option<&TargetData>) -> Option<Action> {
        let action = match target {
            Some(TargetData::Commit(r) | TargetData::Branch(r)) => {
                subscreen_arg(rebase_interactive_cmd, r.into())
            }
            _ => return None,
        };

        Some(action)
    }
    fn is_target_op(&self) -> bool {
        true
    }
}

fn rebase_interactive_cmd(reference: &OsStr) -> Command {
    let mut cmd = Command::new("git");
    cmd.args([
        OsStr::new("rebase"),
        OsStr::new("-i"),
        OsStr::new("--autostash"),
        &parent(reference),
    ]);

    cmd
}

fn parent(reference: &OsStr) -> OsString {
    let mut parent = reference.to_os_string();
    parent.push("^");
    parent
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Rebase autosquash")]
pub(crate) struct RebaseAutosquash;
impl OpTrait for RebaseAutosquash {
    fn get_action(&self, target: Option<&TargetData>) -> Option<Action> {
        let action = match target {
            Some(TargetData::Commit(r) | TargetData::Branch(r)) => {
                subscreen_arg(rebase_autosquash_cmd, r.into())
            }
            _ => return None,
        };

        Some(action)
    }
    fn is_target_op(&self) -> bool {
        true
    }
}

fn rebase_autosquash_cmd(reference: &OsStr) -> Command {
    let mut cmd = Command::new("git");
    cmd.args([
        OsStr::new("rebase"),
        OsStr::new("-i"),
        OsStr::new("--autosquash"),
        OsStr::new("--keep-empty"),
        OsStr::new("--autostash"),
        &reference,
    ]);
    cmd
}
