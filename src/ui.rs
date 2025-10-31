use std::borrow::Cow;

use crate::app::State;
use crate::screen;
use crate::ui::layout::LayoutItem;
use layout::LayoutTree;
use layout::OPTS;
use ratatui::Frame;
use ratatui::prelude::*;
use ratatui::style::Stylize;
use ratatui::widgets::Clear;
use tui_prompts::State as _;
use unicode_segmentation::UnicodeSegmentation;

pub(crate) mod layout;
mod menu;

const CARET: &str = "\u{2588}";

#[derive(Debug, Clone)]
pub(crate) enum UiTreeNode<'a> {
    Span((Cow<'a, str>, Style)),
    Clear,
}
pub(crate) type UiTree<'a> = LayoutTree<UiTreeNode<'a>>;

pub(crate) fn ui(frame: &mut Frame, state: &mut State) {
    let mut layout = UiTree::new();

    layout.stacked(None, OPTS, |layout| {
        screen::layout_screen(
            layout,
            frame.area().as_size(),
            state.screens.last().unwrap(),
        );

        layout.vertical(Some(UiTreeNode::Clear), OPTS.align_end(), |layout| {
            menu::layout_menu(layout, state);
            layout_command_log(layout, state);
            layout_prompt(layout, state);
        });
    });

    layout.compute([frame.area().width, frame.area().height]);

    for item in layout.iter() {
        let LayoutItem { data, pos, size } = item;
        let area = Rect::new(pos[0], pos[1], size[0], size[1]);
        match data {
            UiTreeNode::Span((text, style)) => frame.render_widget(SpanRef(text, *style), area),
            UiTreeNode::Clear => frame.render_widget(Clear, area),
        };
    }

    layout.clear();

    state.screens.last_mut().unwrap().size = frame.area().as_size();
}

struct SpanRef<'a>(&'a Cow<'a, str>, Style);

impl<'a> Widget for SpanRef<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let SpanRef(text, style) = self;
        buf.set_string(area.x, area.y, text, style);
    }
}

fn layout_command_log<'a>(layout: &mut UiTree<'a>, state: &State) {
    if !state.current_cmd_log.is_empty() {
        layout_text(layout, state.current_cmd_log.format_log(&state.config));
    }
}

fn layout_prompt<'a>(layout: &mut UiTree<'a>, state: &'a State) {
    let Some(ref prompt_data) = state.prompt.data else {
        return;
    };

    let prompt_symbol = state.prompt.state.status().symbol();

    layout.horizontal(None, OPTS, |layout| {
        layout_span(layout, (prompt_symbol.content, prompt_symbol.style));
        layout_span(layout, (" ".into(), Style::new()));
        layout_span(
            layout,
            (prompt_data.prompt_text.as_ref().into(), Style::new()),
        );
        layout_span(layout, (" › ".into(), Style::new().cyan().dim()));
        layout_span(layout, (state.prompt.state.value().into(), Style::new()));
        layout_span(layout, (CARET.into(), Style::new()));
    });
}

pub(crate) fn layout_text<'a>(layout: &mut UiTree<'a>, text: Text<'a>) {
    layout.vertical(None, OPTS, |layout| {
        for line in text {
            layout_line(layout, line);
        }
    });
}

pub(crate) fn layout_line<'a>(layout: &mut UiTree<'a>, line: Line<'a>) {
    layout.horizontal(None, OPTS, |layout| {
        for span in line {
            layout_span(layout, (span.content, span.style));
        }
    });
}

pub(crate) fn layout_span<'a>(layout: &mut UiTree<'a>, span: (Cow<'a, str>, Style)) {
    let width = span.0.graphemes(true).count() as u16;
    layout.leaf_with_size(UiTreeNode::Span(span), [width, 1]);
}
