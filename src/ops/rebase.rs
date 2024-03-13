use super::{subscreen_arg, Action, OpTrait, TargetOpTrait};
use crate::{git, items::TargetData, state::State, Res};
use ratatui::{backend::Backend, prelude::Terminal};
use std::process::Command;

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct RebaseContinue;
impl<B: Backend> OpTrait<B> for RebaseContinue {
    fn trigger(&self, state: &mut State, term: &mut Terminal<B>) -> Res<()> {
        let mut cmd = Command::new("git");
        cmd.args(["rebase", "--continue"]);

        state.issue_subscreen_command(term, cmd)?;
        Ok(())
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct RebaseAbort;
impl<B: Backend> OpTrait<B> for RebaseAbort {
    fn trigger(&self, state: &mut State, term: &mut Terminal<B>) -> Res<()> {
        let mut cmd = Command::new("git");
        cmd.args(["rebase", "--abort"]);

        state.run_external_cmd(term, &[], cmd)?;
        Ok(())
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct RebaseInteractive;
impl<B: Backend> TargetOpTrait<B> for RebaseInteractive {
    fn get_action(&self, target: TargetData) -> Option<Action<B>> {
        match target {
            TargetData::Commit(r) | TargetData::Branch(r) => {
                subscreen_arg(git::rebase_interactive_cmd, r.into())
            }
            _ => None,
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct RebaseAutosquash;
impl<B: Backend> TargetOpTrait<B> for RebaseAutosquash {
    fn get_action(&self, target: TargetData) -> Option<Action<B>> {
        match target {
            TargetData::Commit(r) | TargetData::Branch(r) => {
                subscreen_arg(git::rebase_autosquash_cmd, r.into())
            }
            _ => None,
        }
    }
}
