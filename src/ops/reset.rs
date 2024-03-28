use super::{cmd_arg, OpTrait};
use crate::{items::TargetData, Action};
use derive_more::Display;
use std::{ffi::OsStr, process::Command};

pub(crate) fn args() -> &'static [(&'static str, bool)] {
    &[]
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Reset soft")]
pub(crate) struct ResetSoft;
impl OpTrait for ResetSoft {
    fn get_action(&self, target: Option<&TargetData>) -> Option<Action> {
        let action = match target {
            Some(TargetData::Commit(r) | TargetData::Branch(r)) => {
                cmd_arg(reset_soft_cmd, r.into())
            }
            _ => return None,
        };

        Some(action)
    }
    fn is_target_op(&self) -> bool {
        true
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Reset mixed")]
pub(crate) struct ResetMixed;
impl OpTrait for ResetMixed {
    fn get_action(&self, target: Option<&TargetData>) -> Option<Action> {
        let action = match target {
            Some(TargetData::Commit(r) | TargetData::Branch(r)) => {
                cmd_arg(reset_mixed_cmd, r.into())
            }
            _ => return None,
        };

        Some(action)
    }
    fn is_target_op(&self) -> bool {
        true
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Reset hard")]
pub(crate) struct ResetHard;
impl OpTrait for ResetHard {
    fn get_action(&self, target: Option<&TargetData>) -> Option<Action> {
        let action = match target {
            Some(TargetData::Commit(r) | TargetData::Branch(r)) => {
                cmd_arg(reset_hard_cmd, r.into())
            }
            _ => return None,
        };

        Some(action)
    }
    fn is_target_op(&self) -> bool {
        true
    }
}

fn reset_soft_cmd(reference: &OsStr) -> Command {
    let mut cmd = Command::new("git");
    cmd.args(["reset", "--soft"]);
    cmd.arg(reference);
    cmd
}

fn reset_mixed_cmd(reference: &OsStr) -> Command {
    let mut cmd = Command::new("git");
    cmd.args(["reset", "--mixed"]);
    cmd.arg(reference);
    cmd
}

fn reset_hard_cmd(reference: &OsStr) -> Command {
    let mut cmd = Command::new("git");
    cmd.args(["reset", "--hard"]);
    cmd.arg(reference);
    cmd
}
