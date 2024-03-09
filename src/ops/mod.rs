use ratatui::backend::Backend;

use crate::{state::State, Res};
use std::borrow::Cow;

pub(crate) mod checkout;
pub(crate) mod discard;

pub(crate) trait OpTrait<B: Backend> {
    fn trigger(&self, state: &mut State) -> Res<()>;
    fn format_prompt(&self) -> Cow<'static, str>;
    fn prompt_update(
        &self,
        status: tui_prompts::prelude::Status,
        state: &mut State,
        term: &mut ratatui::prelude::Terminal<B>,
    ) -> Res<()>;
}
