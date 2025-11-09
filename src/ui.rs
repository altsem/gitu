use std::borrow::Cow;

use crate::app::State;
use crate::screen;
use crate::ui::layout::LayoutItem;
use itertools::Itertools;
use layout::LayoutTree;
use layout::OPTS;
use ratatui::Frame;
use ratatui::prelude::*;
use ratatui::style::Stylize;
use tui_prompts::State as _;
use unicode_segmentation::UnicodeSegmentation;

pub(crate) mod layout;
mod menu;

const CARET: &str = "\u{2588}";
const DASHES: &str = "────────────────────────────────────────────────────────────────";
const STYLE: Style = Style {
    fg: None,
    bg: None,
    underline_color: None,
    add_modifier: Modifier::DIM,
    sub_modifier: Modifier::empty(),
};

pub(crate) type UiTree<'a> = LayoutTree<(Cow<'a, str>, Style)>;

pub(crate) fn ui(frame: &mut Frame, state: &mut State) {
    let size = frame.area().as_size();
    let mut layout = UiTree::new();

    layout.vertical(None, OPTS, |layout| {
        layout.vertical(None, OPTS.grow(), |layout| {
            screen::layout_screen(layout, size, state.screens.last().unwrap());
        });

        layout.vertical(None, OPTS, |layout| {
            menu::layout_menu(layout, state, size.width as usize);
            layout_command_log(layout, state, size.width as usize);
            layout_prompt(layout, state, size.width as usize);
        });
    });

    layout.compute([frame.area().width, frame.area().height]);

    for item in layout.iter() {
        let LayoutItem { data, pos, size } = item;
        let area = Rect::new(pos[0], pos[1], size[0], size[1]);
        let (text, style) = data;
        frame.render_widget(SpanRef(text, *style), area);
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

fn layout_command_log<'a>(layout: &mut UiTree<'a>, state: &State, width: usize) {
    if !state.current_cmd_log.is_empty() {
        repeat_chars(layout, width, DASHES, STYLE);
        layout_text(layout, state.current_cmd_log.format_log(&state.config));
    }
}

fn layout_prompt<'a>(layout: &mut UiTree<'a>, state: &'a State, width: usize) {
    let Some(ref prompt_data) = state.prompt.data else {
        return;
    };

    let prompt_symbol = state.prompt.state.status().symbol();

    repeat_chars(layout, width, DASHES, STYLE);
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
    layout.leaf_with_size(span, [width, 1]);
}

pub(crate) fn repeat_chars(layout: &mut UiTree, count: usize, chars: &'static str, style: Style) {
    let grapheme_count = chars.grapheme_indices(true).count();
    let full = count / grapheme_count;
    let partial = count % grapheme_count;

    layout.horizontal(None, OPTS, |layout| {
        for _ in 0..full {
            layout_span(layout, (chars.into(), style));
        }

        if partial > 0 {
            let end = chars
                .grapheme_indices(true)
                .tuple_windows()
                .take(partial)
                .last()
                .map(|((_, _), (end, _))| end)
                .unwrap_or(chars.len());

            layout_span(layout, (chars[..end].into(), style));
        }
    });
}
