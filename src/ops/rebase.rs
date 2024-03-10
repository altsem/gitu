use super::OpTrait;
use crate::{state::State, Res};
use ratatui::{backend::Backend, prelude::Terminal};
use std::process::Command;

pub(crate) struct RebaseContinue {}

impl<B: Backend> OpTrait<B> for RebaseContinue {
    fn trigger(&self, state: &mut State, term: &mut Terminal<B>) -> Res<()> {
        let mut cmd = Command::new("git");
        cmd.args(["rebase", "--continue"]);

        state.issue_subscreen_command(term, cmd)?;
        Ok(())
    }
}

pub(crate) struct RebaseAbort {}

impl<B: Backend> OpTrait<B> for RebaseAbort {
    fn trigger(&self, state: &mut State, term: &mut Terminal<B>) -> Res<()> {
        let mut cmd = Command::new("git");
        cmd.args(["rebase", "--abort"]);

        state.run_external_cmd(term, &[], cmd)?;
        Ok(())
    }
}
