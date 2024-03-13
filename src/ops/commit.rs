use super::{subscreen_arg, Action, OpTrait, TargetOpTrait};
use crate::{git, items::TargetData, state::State, Res};
use ratatui::{backend::Backend, prelude::Terminal};
use std::process::Command;

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Commit;
impl<B: Backend> OpTrait<B> for Commit {
    fn trigger(&self, state: &mut State, term: &mut Terminal<B>) -> Res<()> {
        let mut cmd = Command::new("git");
        cmd.args(["commit"]);

        state.issue_subscreen_command(term, cmd)?;
        Ok(())
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct CommitAmend;
impl<B: Backend> OpTrait<B> for CommitAmend {
    fn trigger(&self, state: &mut State, term: &mut Terminal<B>) -> Res<()> {
        let mut cmd = Command::new("git");
        cmd.args(["commit", "--amend"]);

        state.issue_subscreen_command(term, cmd)?;
        Ok(())
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct CommitFixup;
impl<B: Backend> TargetOpTrait<B> for CommitFixup {
    fn get_action(&self, target: TargetData) -> Option<Action<B>> {
        match target {
            TargetData::Commit(r) => subscreen_arg(git::commit_fixup_cmd, r.into()),
            _ => None,
        }
    }
}
