use std::borrow::Cow;

use ratatui::prelude::*;
use tui_prompts::State as _;
use unicode_segmentation::UnicodeSegmentation;

use crate::config::Config;
use crate::picker::PickerState;
use crate::ui::layout::OPTS;
use crate::ui::{CARET, DASHES, STYLE, UiTree, layout_span, repeat_chars};

const MAX_ITEMS_DISPLAY: usize = 10;

/// Layout the picker UI
pub(crate) fn layout_picker<'a>(
    layout: &mut UiTree<'a>,
    state: &'a PickerState,
    config: &Config,
    width: usize,
) {
    // Separator line
    repeat_chars(layout, width, DASHES, STYLE);

    let prompt_style: Style = (&config.style.picker.prompt).into();
    let info_style: Style = (&config.style.picker.info).into();
    layout.horizontal(None, OPTS, |layout| {
        let status_text = format!(" {}/{}   ", state.filtered_count(), state.total_items());
        layout_span(layout, (status_text.into(), info_style));

        // Prompt with separator (like regular prompt)
        layout_span(layout, (state.prompt_text.as_ref().into(), prompt_style));
        layout_span(layout, (" › ".into(), prompt_style));
        layout_span(layout, (state.input_state.value().into(), Style::new()));
        layout_span(layout, (CARET.into(), Style::new()));
    });

    // Calculate visible items range (scroll window)
    let cursor = state.cursor();
    let total_items = state.filtered_items().count();
    let visible_count = MAX_ITEMS_DISPLAY.min(total_items);
    let start = calculate_visible_range(cursor, total_items, MAX_ITEMS_DISPLAY);

    // Render items - always show MAX_ITEMS_DISPLAY rows for fixed height
    let mut rendered_count = 0;
    for (display_idx, (original_idx, item)) in state
        .filtered_items()
        .enumerate()
        .skip(start)
        .take(visible_count)
    {
        let is_selected = display_idx == cursor;
        let style = if is_selected {
            (&config.style.picker.selection_line).into()
        } else {
            Style::new()
        };

        layout.horizontal(None, OPTS, |layout| {
            // Selection indicator (cursor bar like status screen)
            if is_selected {
                let cursor_style: Style = (&config.style.cursor).into();
                let indicator = format!("{}", config.style.cursor.symbol);
                layout_span(layout, (indicator.into(), cursor_style));
            } else {
                layout_span(layout, (" ".into(), Style::new()));
            }

            // Render item text with fuzzy match highlighting
            if let Some(match_indices) = state.match_indices(original_idx) {
                render_highlighted_text(layout, &item.display, &match_indices, style, config);
            } else {
                layout_span(layout, (item.display.as_ref().into(), style));
            }
        });
        rendered_count += 1;
    }

    // Fill remaining rows with empty lines to maintain fixed height
    for _ in rendered_count..MAX_ITEMS_DISPLAY {
        layout.horizontal(None, OPTS, |layout| {
            layout_span(layout, (" ".into(), Style::new()));
        });
    }
}

/// Calculate the visible range start for scrolling based on cursor position
fn calculate_visible_range(cursor: usize, total: usize, max_items: usize) -> usize {
    if total <= max_items {
        return 0;
    }

    // Center the cursor in the visible window
    let half = max_items / 2;
    if cursor < half {
        0
    } else if cursor >= total - half {
        total - max_items
    } else {
        cursor - half
    }
}

/// Render text with specific characters highlighted (for fuzzy match visualization)
fn render_highlighted_text<'a>(
    layout: &mut UiTree<'a>,
    text: &'a str,
    highlight_indices: &[usize],
    base_style: Style,
    config: &Config,
) {
    let graphemes: Vec<&str> = text.graphemes(true).collect();
    let highlight_style: Style = (&config.style.picker.matched).into();

    let mut buffer = String::new();

    for (idx, &grapheme) in graphemes.iter().enumerate() {
        let should_highlight = highlight_indices.contains(&idx);

        if should_highlight {
            // Flush non-highlighted buffer
            if !buffer.is_empty() {
                layout_span(layout, (Cow::Owned(buffer.clone()), base_style));
                buffer.clear();
            }
            // Render highlighted character
            layout_span(layout, (Cow::Owned(grapheme.to_string()), highlight_style));
        } else {
            buffer.push_str(grapheme);
        }
    }

    // Flush remaining buffer
    if !buffer.is_empty() {
        layout_span(layout, (Cow::Owned(buffer), base_style));
    }
}
