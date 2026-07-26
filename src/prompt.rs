use crate::text_input::TextInput;
use std::borrow::Cow;

pub(crate) struct PromptData {
    pub(crate) prompt_text: Cow<'static, str>,
}

pub(crate) struct Prompt {
    pub(crate) data: Option<PromptData>,
    pub(crate) state: TextInput,
}

impl Prompt {
    pub(crate) fn new() -> Self {
        Prompt {
            data: None,
            state: TextInput::default(),
        }
    }

    pub(crate) fn set(&mut self, data: PromptData) {
        self.data = Some(data);
        self.state.focused = true;
    }

    pub(crate) fn reset(&mut self) {
        self.data = None;
        self.state = TextInput::default();
    }
}
