use ratatui::{backend::Backend, prelude::Terminal};

use crate::{git, state::State, Res};

use super::OpTrait;

pub(crate) struct Commit {}

impl<B: Backend> OpTrait<B> for Commit {
    fn trigger(&self, state: &mut State, term: &mut Terminal<B>) -> Res<()> {
        state.issue_subscreen_command(term, git::commit_cmd())?;
        Ok(())
    }
}

pub(crate) struct CommitAmend {}

impl<B: Backend> OpTrait<B> for CommitAmend {
    fn trigger(&self, state: &mut State, term: &mut Terminal<B>) -> Res<()> {
        state.issue_subscreen_command(term, git::commit_amend_cmd())?;
        Ok(())
    }
}
