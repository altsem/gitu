use crate::ops::Op;

use super::Res;
use ratatui::{backend::Backend, Terminal};
use tui_prompts::{State, TextState};

pub(crate) struct Prompt {
    pub(crate) pending_op: Option<Op>,
    pub(crate) state: TextState<'static>,
}

impl Prompt {
    pub(crate) fn new() -> Self {
        Prompt {
            pending_op: None,
            state: TextState::new(),
        }
    }

    pub(crate) fn set(&mut self, op: Op) {
        self.pending_op = Some(op);
        self.state.focus();
    }

    pub(crate) fn reset<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Res<()> {
        self.pending_op = None;
        self.state = TextState::new();
        terminal.hide_cursor()?;
        Ok(())
    }
}
