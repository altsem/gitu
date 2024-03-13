use super::{cmd_arg, TargetOpTrait};
use crate::{git, items::TargetData, Action};

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct ResetSoft;
impl TargetOpTrait for ResetSoft {
    fn get_action(&self, target: TargetData) -> Option<Action> {
        match target {
            TargetData::Commit(r) | TargetData::Branch(r) => cmd_arg(git::reset_soft_cmd, r.into()),
            _ => None,
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct ResetMixed;
impl TargetOpTrait for ResetMixed {
    fn get_action(&self, target: TargetData) -> Option<Action> {
        match target {
            TargetData::Commit(r) | TargetData::Branch(r) => {
                cmd_arg(git::reset_mixed_cmd, r.into())
            }
            _ => None,
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct ResetHard;
impl TargetOpTrait for ResetHard {
    fn get_action(&self, target: TargetData) -> Option<Action> {
        match target {
            TargetData::Commit(r) | TargetData::Branch(r) => cmd_arg(git::reset_hard_cmd, r.into()),
            _ => None,
        }
    }
}
