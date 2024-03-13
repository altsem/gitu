use super::{subscreen_arg, Action, OpTrait, TargetOpTrait};
use crate::{items::TargetData, state::State, term::Term, Res};
use derive_more::Display;
use std::{
    ffi::{OsStr, OsString},
    process::Command,
};

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Rebase continue")]
pub(crate) struct RebaseContinue;
impl OpTrait for RebaseContinue {
    fn trigger(&self, state: &mut State, term: &mut Term) -> Res<()> {
        let mut cmd = Command::new("git");
        cmd.args(["rebase", "--continue"]);

        state.issue_subscreen_command(term, cmd)?;
        Ok(())
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Rebase abort")]
pub(crate) struct RebaseAbort;
impl OpTrait for RebaseAbort {
    fn trigger(&self, state: &mut State, term: &mut Term) -> Res<()> {
        let mut cmd = Command::new("git");
        cmd.args(["rebase", "--abort"]);

        state.run_external_cmd(term, &[], cmd)?;
        Ok(())
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Rebase interactive")]
pub(crate) struct RebaseInteractive;
impl TargetOpTrait for RebaseInteractive {
    fn get_action(&self, target: TargetData) -> Option<Action> {
        match target {
            TargetData::Commit(r) | TargetData::Branch(r) => {
                subscreen_arg(rebase_interactive_cmd, r.into())
            }
            _ => None,
        }
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
impl TargetOpTrait for RebaseAutosquash {
    fn get_action(&self, target: TargetData) -> Option<Action> {
        match target {
            TargetData::Commit(r) | TargetData::Branch(r) => {
                subscreen_arg(rebase_autosquash_cmd, r.into())
            }
            _ => None,
        }
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
