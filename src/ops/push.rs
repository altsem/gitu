use super::OpTrait;
use crate::{state::State, Res};
use ratatui::{backend::Backend, prelude::Terminal};
use std::process::Command;

pub(crate) struct Push;
impl<B: Backend> OpTrait<B> for Push {
    fn trigger(&self, state: &mut State, term: &mut Terminal<B>) -> Res<()> {
        let mut cmd = Command::new("git");
        cmd.args(["push"]);

        state.run_external_cmd(term, &[], cmd)?;
        Ok(())
    }
}
