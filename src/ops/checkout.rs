use super::{Op, OpTrait};
use crate::{state::State, Res};
use ratatui::{backend::Backend, prelude::Terminal};
use std::{borrow::Cow, process::Command};
use tui_prompts::{prelude::Status, State as _};

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct CheckoutNewBranch;
impl<B: Backend> OpTrait<B> for CheckoutNewBranch {
    fn trigger(&self, state: &mut State, _term: &mut Terminal<B>) -> Res<()> {
        state.prompt.set(Op::CheckoutNewBranch(CheckoutNewBranch));
        Ok(())
    }

    fn format_prompt(&self) -> Cow<'static, str> {
        "Create and checkout branch:".into()
    }

    fn prompt_update(&self, status: Status, state: &mut State, term: &mut Terminal<B>) -> Res<()> {
        if status.is_done() {
            let name = state.prompt.state.value().to_string();
            let mut cmd = Command::new("git");
            cmd.args(["checkout", "-b", &name]);

            state.run_external_cmd(term, &[], cmd)?;
            state.prompt.reset(term)?;
        }
        Ok(())
    }
}
