use super::OpTrait;
use crate::{
    get_action,
    keybinds::{Op, TargetOp},
    state::State,
    ErrorBuffer, Res,
};
use ratatui::{backend::Backend, Terminal};
use std::borrow::Cow;
use tui_prompts::{prelude::Status, State as _};

pub(crate) struct Discard {}

impl<B: Backend> OpTrait<B> for Discard {
    fn trigger(&self, state: &mut State, _term: &mut Terminal<B>) -> Res<()> {
        state.prompt_action::<B>(Op::Target(TargetOp::Discard));
        Ok(())
    }

    fn format_prompt(&self) -> Cow<'static, str> {
        // TODO Show what is being discarded
        "Really discard? (y or n)".into()
    }

    fn prompt_update(&self, status: Status, state: &mut State, term: &mut Terminal<B>) -> Res<()> {
        if status.is_pending() {
            match state.prompt.state.value() {
                "y" => {
                    let mut action =
                        get_action(state.clone_target_data(), TargetOp::Discard).unwrap();
                    action(state, term)?;
                    state.prompt.reset(term)?;
                }
                "" => (),
                _ => {
                    state.error_buffer = Some(ErrorBuffer("Discard aborted".to_string()));
                    state.prompt.reset(term)?;
                }
            }
        }
        Ok(())
    }
}
