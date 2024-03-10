use std::borrow::Cow;

use ratatui::{backend::Backend, prelude::Terminal};
use tui_prompts::prelude::Status;

use crate::{state::State, Res};

pub(crate) mod checkout;
pub(crate) mod commit;
pub(crate) mod discard;
pub(crate) mod log;

pub(crate) trait OpTrait<B: Backend> {
    fn trigger(&self, state: &mut State, term: &mut Terminal<B>) -> Res<()>;

    fn format_prompt(&self) -> Cow<'static, str> {
        unimplemented!()
    }

    fn prompt_update(
        &self,
        _status: Status,
        _state: &mut State,
        _term: &mut Terminal<B>,
    ) -> Res<()> {
        unimplemented!()
    }
}
