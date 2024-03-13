use super::{subscreen_arg, Action, OpTrait, TargetOpTrait};
use crate::{git, items::TargetData, state::State, term::Term, Res};
use std::process::Command;

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Commit;
impl OpTrait for Commit {
    fn trigger(&self, state: &mut State, term: &mut Term) -> Res<()> {
        let mut cmd = Command::new("git");
        cmd.args(["commit"]);

        state.issue_subscreen_command(term, cmd)?;
        Ok(())
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct CommitAmend;
impl OpTrait for CommitAmend {
    fn trigger(&self, state: &mut State, term: &mut Term) -> Res<()> {
        let mut cmd = Command::new("git");
        cmd.args(["commit", "--amend"]);

        state.issue_subscreen_command(term, cmd)?;
        Ok(())
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct CommitFixup;
impl TargetOpTrait for CommitFixup {
    fn get_action(&self, target: TargetData) -> Option<Action> {
        match target {
            TargetData::Commit(r) => subscreen_arg(git::commit_fixup_cmd, r.into()),
            _ => None,
        }
    }
}
