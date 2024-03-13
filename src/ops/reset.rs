use super::{cmd_arg, TargetOpTrait};
use crate::{git, items::TargetData, Action};
use ratatui::backend::Backend;

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct ResetSoft;
impl<B: Backend> TargetOpTrait<B> for ResetSoft {
    fn get_action(&self, target: TargetData) -> Option<Action<B>> {
        match target {
            TargetData::Commit(r) | TargetData::Branch(r) => cmd_arg(git::reset_soft_cmd, r.into()),
            _ => None,
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct ResetMixed;
impl<B: Backend> TargetOpTrait<B> for ResetMixed {
    fn get_action(&self, target: TargetData) -> Option<Action<B>> {
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
impl<B: Backend> TargetOpTrait<B> for ResetHard {
    fn get_action(&self, target: TargetData) -> Option<Action<B>> {
        match target {
            TargetData::Commit(r) | TargetData::Branch(r) => cmd_arg(git::reset_hard_cmd, r.into()),
            _ => None,
        }
    }
}
