use super::OpTrait;
use crate::{
    get_action,
    keybinds::{Op, TargetOp},
    ErrorBuffer,
};
use ratatui::backend::Backend;
use std::borrow::Cow;
use tui_prompts::State;

pub(crate) struct Discard {}

impl<B: Backend> OpTrait<B> for Discard {
    fn trigger(&self, state: &mut crate::state::State) -> crate::Res<()> {
        state.prompt_action::<B>(Op::Target(TargetOp::Discard));
        Ok(())
    }

    fn format_prompt(&self) -> Cow<'static, str> {
        // TODO Show what is being discarded
        "Really discard? (y or n)".into()
    }

    fn prompt_update(
        &self,
        status: tui_prompts::prelude::Status,
        state: &mut crate::state::State,
        term: &mut ratatui::prelude::Terminal<B>,
    ) -> crate::Res<()> {
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
