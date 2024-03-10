use super::OpTrait;
use crate::state::State;
use ratatui::{backend::Backend, prelude::Terminal};
use std::process::Command;

pub(crate) struct FetchAll {}

impl<B: Backend> OpTrait<B> for FetchAll {
    fn trigger(&self, state: &mut State, term: &mut Terminal<B>) -> crate::Res<()> {
        let mut cmd = Command::new("git");
        cmd.args(["fetch", "--all"]);

        state.run_external_cmd(term, &[], cmd)?;
        Ok(())
    }
}
