use super::OpTrait;
use crate::{cmd_arg, git, keybinds::Op, state::State, Res};
use ratatui::{backend::Backend, prelude::Terminal};
use std::borrow::Cow;
use tui_prompts::{prelude::Status, State as _};

pub(crate) struct CheckoutNewBranch {}

impl<B: Backend> OpTrait<B> for CheckoutNewBranch {
    fn trigger(&self, state: &mut State, _term: &mut Terminal<B>) -> Res<()> {
        state.prompt.set(Op::CheckoutNewBranch);
        Ok(())
    }

    fn format_prompt(&self) -> Cow<'static, str> {
        "Create and checkout branch:".into()
    }

    fn prompt_update(&self, status: Status, state: &mut State, term: &mut Terminal<B>) -> Res<()> {
        if status.is_done() {
            let name = state.prompt.state.value().to_string();
            cmd_arg(git::checkout_new_branch_cmd, name.into()).unwrap()(state, term)?;
            state.prompt.reset(term)?;
        }
        Ok(())
    }
}
