use crate::app::State;
use ratatui::prelude::*;
use ratatui::style::Stylize;
use ratatui::widgets::*;
use ratatui::Frame;
use std::rc::Rc;
use tui_prompts::State as _;
use tui_prompts::TextPrompt;

mod menu;

pub(crate) struct SizedWidget<W> {
    height: u16,
    widget: W,
}

impl<W: Widget> Widget for SizedWidget<W> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.widget.render(area, buf);
    }
}

impl<W: StatefulWidget> StatefulWidget for SizedWidget<W> {
    type State = W::State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.widget.render(area, buf, state);
    }
}

pub(crate) fn ui(frame: &mut Frame, state: &mut State) {
    let maybe_log = if !state.current_cmd_log.is_empty() {
        let text: Text = state.current_cmd_log.format_log(&state.config);

        Some(SizedWidget {
            widget: Paragraph::new(text.clone()).block(popup_block()),
            height: 1 + text.lines.len() as u16,
        })
    } else {
        None
    };

    let maybe_prompt = state.prompt.data.as_ref().map(|prompt_data| SizedWidget {
        height: 2,
        widget: TextPrompt::new(prompt_data.prompt_text.clone()).with_block(popup_block()),
    });

    let maybe_menu = state.pending_menu.as_ref().and_then(|menu| {
        if menu.is_hidden {
            None
        } else {
            Some(menu::MenuWidget::new(
                Rc::clone(&state.config),
                menu,
                state.screens.last().unwrap().get_selected_item(),
                state,
            ))
        }
    });

    let layout = Layout::new(
        Direction::Vertical,
        [
            Constraint::Min(1),
            widget_height(&maybe_prompt),
            widget_height(&maybe_menu),
            widget_height(&maybe_log),
        ],
    )
    .split(frame.area());

    frame.render_widget(state.screens.last().unwrap(), layout[0]);

    maybe_render(maybe_menu, frame, layout[2]);
    maybe_render(maybe_log, frame, layout[3]);

    if let Some(prompt) = maybe_prompt {
        frame.render_stateful_widget(prompt, layout[1], &mut state.prompt.state);
        let (cx, cy) = state.prompt.state.cursor();
        frame.set_cursor_position((cx, cy));
    }

    state.screens.last_mut().unwrap().size = layout[0].as_size();
}

fn popup_block() -> Block<'static> {
    Block::new()
        .borders(Borders::TOP)
        .border_style(Style::new().dim())
        .border_type(ratatui::widgets::BorderType::Plain)
}

fn widget_height<W>(maybe_prompt: &Option<SizedWidget<W>>) -> Constraint {
    Constraint::Length(
        maybe_prompt
            .as_ref()
            .map(|widget| widget.height)
            .unwrap_or(0),
    )
}

fn maybe_render<W: Widget>(maybe_menu: Option<SizedWidget<W>>, frame: &mut Frame, area: Rect) {
    if let Some(menu) = maybe_menu {
        frame.render_widget(menu, area);
    }
}
