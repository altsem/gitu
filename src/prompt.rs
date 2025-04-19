use crate::ops::Action;
use std::borrow::Cow;
use tui_prompts::{State as _, TextState};

pub(crate) struct PromptData {
    pub(crate) prompt_text: Cow<'static, str>,
    pub(crate) update_fn: Action,
}

pub(crate) struct Prompt {
    pub(crate) data: Option<PromptData>,
    pub(crate) state: TextState<'static>,
}

impl Prompt {
    pub(crate) fn new() -> Self {
        Prompt {
            data: None,
            state: TextState::new(),
        }
    }

    pub(crate) fn set(&mut self, data: PromptData) {
        self.data = Some(data);
        self.state.focus();
    }

    pub(crate) fn reset(&mut self) {
        self.data = None;
        self.state = TextState::new();
    }
}
