use super::Res;
use crate::error::Error;
use itertools::chain;
use ratatui::{
    backend::Backend,
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    text::{Line, Span},
    widgets::{Block, Paragraph, StatefulWidget, Widget},
    Terminal,
};
use std::{borrow::Cow, iter::once};
use termwiz::input::{KeyCode, KeyEvent, Modifiers};

pub(crate) struct PromptData {
    pub(crate) prompt_text: Cow<'static, str>,
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

    pub(crate) fn reset<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Res<()> {
        self.data = None;
        self.state = TextState::new();
        terminal.hide_cursor().map_err(Error::Term)?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct TextState<'a> {
    pub status: Status,
    pub focus: FocusState,
    position: usize,
    pub cursor: (u16, u16),
    pub value: Cow<'a, str>,
}

impl TextState<'_> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            status: Status::Pending,
            focus: FocusState::Unfocused,
            position: 0,
            cursor: (0, 0),
            value: Cow::Borrowed(""),
        }
    }

    /// Sets the focus state of the prompt to [`Focus::Focused`].
    pub fn focus(&mut self) {
        self.focus = FocusState::Focused;
    }

    /// Whether the prompt is focused.
    pub fn is_focused(&self) -> bool {
        self.focus == FocusState::Focused
    }

    pub fn len(&self) -> usize {
        self.value.chars().count()
    }

    pub fn handle_key_event(&mut self, key_event: &KeyEvent) {
        match (key_event.key, key_event.modifiers) {
            (KeyCode::Enter, _) => self.complete(),
            (KeyCode::Escape, _) | (KeyCode::Char('c'), Modifiers::CTRL) => self.abort(),
            (KeyCode::LeftArrow, _) | (KeyCode::Char('b'), Modifiers::CTRL) => self.move_left(),
            (KeyCode::RightArrow, _) | (KeyCode::Char('f'), Modifiers::CTRL) => self.move_right(),
            (KeyCode::Home, _) | (KeyCode::Char('a'), Modifiers::CTRL) => self.move_start(),
            (KeyCode::End, _) | (KeyCode::Char('e'), Modifiers::CTRL) => self.move_end(),
            (KeyCode::Backspace, _) | (KeyCode::Char('h'), Modifiers::CTRL) => {
                self.backspace();
            }
            (KeyCode::Delete, _) | (KeyCode::Char('d'), Modifiers::CTRL) => self.delete(),
            (KeyCode::Char('k'), Modifiers::CTRL) => self.kill(),
            (KeyCode::Char('u'), Modifiers::CTRL) => self.truncate(),
            (KeyCode::Char(c), Modifiers::NONE) => self.push(c),
            _ => {}
        }
    }

    pub fn complete(&mut self) {
        self.status = Status::Done;
    }

    pub fn abort(&mut self) {
        self.status = Status::Aborted;
    }

    pub fn delete(&mut self) {
        let position = self.position;
        if position == self.len() {
            return;
        }
        self.value = chain!(
            self.value.chars().take(position),
            self.value.chars().skip(position + 1)
        )
        .collect();
    }

    pub fn backspace(&mut self) {
        let position = self.position;
        if position == 0 {
            return;
        }
        self.value = chain!(
            self.value.chars().take(position.saturating_sub(1)),
            self.value.chars().skip(position)
        )
        .collect();
        self.position = position.saturating_sub(1);
    }

    pub fn move_right(&mut self) {
        if self.position == self.len() {
            return;
        }
        self.position = self.position.saturating_add(1);
    }

    pub fn move_left(&mut self) {
        self.position = self.position.saturating_sub(1);
    }

    pub fn move_end(&mut self) {
        self.position = self.len();
    }

    pub fn move_start(&mut self) {
        self.position = 0;
    }

    pub fn kill(&mut self) {
        let position = self.position;
        self.value.to_string().truncate(position);
    }

    pub fn truncate(&mut self) {
        self.value.to_mut().clear();
        self.position = 0;
    }

    pub fn push(&mut self, c: char) {
        if self.position == self.len() {
            self.value.to_mut().push(c);
        } else {
            // We cannot use String::insert() as it operates on bytes, which can lead to incorrect modifications with
            // multibyte characters. Instead, we handle text manipulation at the character level using Rust's char type
            // for Unicode correctness. Check docs of String::insert() and String::chars() for futher info.
            self.value = chain![
                self.value.chars().take(self.position),
                once(c),
                self.value.chars().skip(self.position)
            ]
            .collect();
        }
        self.position = self.position.saturating_add(1);
    }
}

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, Hash)]
pub enum FocusState {
    #[default]
    Unfocused,
    Focused,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Status {
    #[default]
    Pending,
    Aborted,
    Done,
}

impl Status {
    #[must_use]
    pub fn symbol(&self) -> Span<'static> {
        match self {
            Self::Pending => Symbols::default().pending,
            Self::Aborted => Symbols::default().aborted,
            Self::Done => Symbols::default().done,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Symbols {
    pub pending: Span<'static>,
    pub aborted: Span<'static>,
    pub done: Span<'static>,
}

impl Default for Symbols {
    fn default() -> Self {
        Self {
            pending: "?".cyan(),
            aborted: "✘".red(),
            done: "✔".green(),
        }
    }
}

/// A prompt widget that displays a message and a text input.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TextPrompt<'a> {
    /// The message to display to the user before the input.
    message: Cow<'a, str>,
    /// The block to wrap the prompt in.
    block: Option<Block<'a>>,
}

impl<'a> TextPrompt<'a> {
    #[must_use]
    pub const fn new(message: Cow<'a, str>) -> Self {
        Self {
            message,
            block: None,
        }
    }

    #[must_use]
    pub fn with_block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

// impl Prompt for TextPrompt<'_> {
//     /// Draws the prompt widget.
//     ///
//     /// This is in addition to the `Widget` trait implementation as we need the `Frame` to set the
//     /// cursor position.
//     fn draw(self, frame: &mut Frame, area: Rect, state: &mut Self::State) {
//         frame.render_stateful_widget(self, area, state);
//         if state.is_focused() {
//             frame.set_cursor_position(state.cursor());
//         }
//     }
// }

impl<'a> StatefulWidget for TextPrompt<'a> {
    type State = TextState<'a>;

    fn render(mut self, mut area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.render_block(&mut area, buf);

        let width = area.width as usize;
        let height = area.height as usize;
        let value = state.value.clone();
        let value_length = value.chars().count();

        let line = Line::from(vec![
            state.status.symbol(),
            " ".into(),
            self.message.bold(),
            " › ".cyan().dim(),
            Span::raw(value),
        ]);
        let prompt_length = line.width() - value_length;
        let lines = wrap(line, width).take(height).collect::<Vec<_>>();

        // constrain the position to the area
        let position = (state.position + prompt_length).min(area.area() as usize - 1);
        let row = position / width;
        let column = position % width;
        state.cursor = (area.x + column as u16, area.y + row as u16);
        Paragraph::new(lines).render(area, buf);
    }
}

/// wraps a line into multiple lines of the given width.
///
/// This is a character based wrap, not a word based wrap.
///
/// TODO: move this into the `Line` type.
fn wrap(line: Line, width: usize) -> impl Iterator<Item = Line> {
    let mut line = line;
    std::iter::from_fn(move || {
        if line.width() > width {
            let (first, second) = line_split_at(line.clone(), width);
            line = second;
            Some(first)
        } else if line.width() > 0 {
            let first = line.clone();
            line = Line::default();
            Some(first)
        } else {
            None
        }
    })
}

/// splits a line into two lines at the given position.
///
/// TODO: move this into the `Line` type.
/// TODO: fix this so that it operates on multi-width characters.
fn line_split_at(line: Line, mid: usize) -> (Line, Line) {
    let mut first = Line::default();
    let mut second = Line::default();
    first.alignment = line.alignment;
    second.alignment = line.alignment;
    for span in line.spans {
        let first_width = first.width();
        let span_width = span.width();
        if first_width + span_width <= mid {
            first.spans.push(span);
        } else if first_width < mid && first_width + span_width > mid {
            let span_mid = mid - first_width;
            let (span_first, span_second) = span_split_at(span, span_mid);
            first.spans.push(span_first);
            second.spans.push(span_second);
        } else {
            second.spans.push(span);
        }
    }
    (first, second)
}

/// splits a span into two spans at the given position.
///
/// TODO: move this into the `Span` type.
/// TODO: fix this so that it operates on multi-width characters.
fn span_split_at(span: Span, mid: usize) -> (Span, Span) {
    let (first, second) = span.content.split_at(mid);
    let first = Span {
        content: Cow::Owned(first.into()),
        style: span.style,
    };
    let second = Span {
        content: Cow::Owned(second.into()),
        style: span.style,
    };
    (first, second)
}

impl TextPrompt<'_> {
    fn render_block(&mut self, area: &mut Rect, buf: &mut Buffer) {
        if let Some(block) = self.block.take() {
            let inner = block.inner(*area);
            block.render(*area, buf);
            *area = inner;
        };
    }
}

impl<T> From<T> for TextPrompt<'static>
where
    T: Into<Cow<'static, str>>,
{
    fn from(message: T) -> Self {
        Self::new(message.into())
    }
}
