use super::{cmd_arg, OpTrait};
use crate::{git, items::TargetData, Action};
use derive_more::Display;

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
                cmd_arg(git::reset_soft_cmd, r.into())
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
                cmd_arg(git::reset_mixed_cmd, r.into())
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
                cmd_arg(git::reset_hard_cmd, r.into())
            }
            _ => return None,
        };

        Some(action)
    }
    fn is_target_op(&self) -> bool {
        true
    }
}
