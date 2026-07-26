use std::borrow::Cow;

use crate::Res;
use crate::app::State;
use crate::error::Error;
use crate::screen;
use crate::term::TermBackend;
use crate::ui::layout::LayoutItem;
use itertools::Itertools;
use layout::LayoutTree;
use layout::OPTS;
use ratatui::layout::Size;
use ratatui::style::Style;
use tui_prompts::State as _;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

mod cmd_log;
pub(crate) mod item;
pub(crate) mod layout;
mod menu;
pub mod picker;

const CARET: &str = "\u{2588}";
const DASHES: &str = "────────────────────────────────────────────────────────────────";
const BLANKS: &str = "                                                                ";

#[derive(Debug, Clone)]
pub(crate) enum UiItem<'a> {
    Span(Cow<'a, str>, Style),
    Style(Style),
}
pub(crate) type UiTree<'a> = LayoutTree<UiItem<'a>>;

pub(crate) fn ui(term: &mut TermBackend, state: &mut State) -> Res<()> {
    let size = term.size().unwrap();
    let mut layout = UiTree::new();

    layout.vertical(None, OPTS, |layout| {
        layout.vertical(None, OPTS.fill_xy(), |layout| {
            let hide_cursor = state.picker.is_some();
            screen::layout_screen(layout, state.screens.last().unwrap(), hide_cursor);
        });

        layout.vertical(None, OPTS, |layout| {
            menu::layout_menu(layout, state, size.width as usize);
            cmd_log::layout_cmd_log(
                layout,
                &state.current_cmd_log,
                &state.config,
                size.width as usize,
            );
            layout_prompt(layout, state, size.width as usize);
            layout_picker(layout, state, size.width as usize);
            if !state.pending_keys.is_empty() {
                let keys = &state
                    .pending_keys
                    .iter()
                    .map(|(_, k)| k.to_string())
                    .collect::<String>();

                layout_span(layout, (("    ".to_string() + keys).into(), Style::new()));
            }
        });
    });

    layout.compute([size.width, size.height]);

    let mut items = layout.iter().collect::<Vec<_>>();
    items.sort_by_key(|item| [item.pos[1], item.pos[0]]);

    clear_blanks(term, size, items)?;

    term.flush().map_err(Error::Term)?;
    layout.clear();

    state.screens.last_mut().unwrap().size = size;

    Ok(())
}

fn layout_prompt<'a>(layout: &mut UiTree<'a>, state: &'a State, width: usize) {
    let Some(ref prompt_data) = state.prompt.data else {
        return;
    };

    let prompt_symbol = state.prompt.state.status().symbol();
    let separator_style = Style::from(&state.config.style.separator);
    let prompt_style = Style::from(&state.config.style.prompt);

    repeat_chars(layout, width, DASHES, separator_style);
    layout.horizontal(None, OPTS, |layout| {
        layout_span(layout, (prompt_symbol.content, prompt_symbol.style));
        layout_span(layout, (" ".into(), Style::new()));
        layout_span(
            layout,
            (prompt_data.prompt_text.as_ref().into(), prompt_style),
        );
        layout_span(layout, (" › ".into(), prompt_style));
        layout_span(layout, (state.prompt.state.value().into(), Style::new()));
        layout_span(layout, (CARET.into(), Style::new()));
    });
}

fn layout_picker<'a>(layout: &mut UiTree<'a>, state: &'a State, width: usize) {
    if let Some(ref picker_state) = state.picker {
        picker::layout_picker(layout, picker_state, &state.config, width);
    }
}

/// Lays out `content` as a single row of its own.
pub(crate) fn layout_line<'a>(layout: &mut UiTree<'a>, content: Cow<'a, str>, style: Style) {
    layout.horizontal(None, OPTS, |layout| {
        layout_span(layout, (content, style));
    });
}

pub(crate) fn layout_span<'a>(layout: &mut UiTree<'a>, span: (Cow<'a, str>, Style)) {
    match span.0 {
        Cow::Borrowed(s) => {
            for word in words(s) {
                layout.leaf_with_size(
                    UiItem::Span(Cow::Borrowed(word), span.1),
                    [UnicodeWidthStr::width(word) as u16, 1],
                );
            }
        }
        Cow::Owned(s) => {
            for word in words(&s) {
                layout.leaf_with_size(
                    UiItem::Span(Cow::Owned(word.into()), span.1),
                    [UnicodeWidthStr::width(word) as u16, 1],
                );
            }
        }
    }
}

/// Splits into the words to wrap on, dropping line breaks. A span occupies a
/// single row, so a line break would otherwise be printed as-is and shift the
/// rest of the frame.
fn words(text: &str) -> impl Iterator<Item = &str> {
    text.split_word_bounds()
        .filter(|word| !word.bytes().all(|byte| byte == b'\n' || byte == b'\r'))
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

fn clear_blanks(
    term: &mut TermBackend,
    size: Size,
    items: Vec<LayoutItem<&UiItem<'_>>>,
) -> Result<(), Error> {
    let mut at = [0, 0];
    let mut bg = Style::new();
    let mut bg_end = 0;
    for item in items {
        let LayoutItem {
            data,
            pos,
            size: item_size,
        } = item;

        blank_until(term, &mut at, [0, pos[1]], size.width, bg, bg_end)?;

        match data {
            UiItem::Span(text, style) => {
                blank_until(term, &mut at, pos, size.width, bg, bg_end)?;
                term.queue_move_cursor(pos[0], pos[1])?;
                term.queue_print(text, style)?;

                at[0] = pos[0].saturating_add(item_size[0]);
            }
            UiItem::Style(style) => {
                bg = *style;
                bg_end = pos[1].saturating_add(item_size[1]);
            }
        }
    }
    blank_until(term, &mut at, [0, size.height], size.width, bg, bg_end)?;
    Ok(())
}

fn blank_until(
    term: &mut TermBackend,
    at: &mut [u16; 2],
    to: [u16; 2],
    width: u16,
    bg: Style,
    bg_end: u16,
) -> Res<()> {
    let row_bg = |y| if y < bg_end { bg } else { Style::new() };

    while at[1] < to[1] {
        queue_blanks(term, *at, width.saturating_sub(at[0]), &row_bg(at[1]))?;
        *at = [0, at[1] + 1];
    }

    queue_blanks(term, *at, to[0].saturating_sub(at[0]), &row_bg(at[1]))?;
    at[0] = at[0].max(to[0]);

    Ok(())
}

fn queue_blanks(term: &mut TermBackend, at: [u16; 2], width: u16, style: &Style) -> Res<()> {
    if width == 0 {
        return Ok(());
    }

    term.queue_move_cursor(at[0], at[1])?;

    let mut left = width as usize;
    while left > 0 {
        let blanks = left.min(BLANKS.len());
        term.queue_print(&BLANKS[..blanks], style)?;
        left -= blanks;
    }

    Ok(())
}
