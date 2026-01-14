use std::borrow::Cow;

use ratatui::prelude::*;
use tui_prompts::State as _;
use unicode_segmentation::UnicodeSegmentation;

use crate::config::Config;
use crate::picker::PickerState;
use crate::ui::layout::OPTS;
use crate::ui::{CARET, DASHES, UiTree, layout_span, repeat_chars};

const MAX_ITEMS_DISPLAY: usize = 10;

/// Layout the picker UI
pub(crate) fn layout_picker<'a>(
    layout: &mut UiTree<'a>,
    state: &'a PickerState,
    config: &Config,
    width: usize,
) {
    // Separator line
    let separator_style = Style::from(&config.style.separator);
    repeat_chars(layout, width, DASHES, separator_style);

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
    let highlight_style: Style = (&config.style.picker.matched).into();

    let mut buffer = String::new();

    for (idx, grapheme) in text.graphemes(true).enumerate() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::picker::{PickerData, PickerItem, PickerState};
    use crate::ui::layout::LayoutTree;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use itertools::Itertools;
    use std::collections::BTreeMap;

    /// Create a default test config for picker tests
    fn test_config() -> Config {
        use crate::config::{GeneralConfig, PickerBindingsConfig, StyleConfig};

        Config {
            general: GeneralConfig::default(),
            style: StyleConfig::default(),
            bindings: BTreeMap::new().try_into().unwrap(),
            picker_bindings: PickerBindingsConfig::default().try_into().unwrap(),
        }
    }

    fn create_test_items() -> Vec<PickerItem> {
        vec![
            PickerItem::new("main", PickerData::Revision("main".to_string())),
            PickerItem::new("develop", PickerData::Revision("develop".to_string())),
            PickerItem::new(
                "feature/test",
                PickerData::Revision("feature/test".to_string()),
            ),
            PickerItem::new(
                "feature/new",
                PickerData::Revision("feature/new".to_string()),
            ),
            PickerItem::new("bugfix/123", PickerData::Revision("bugfix/123".to_string())),
        ]
    }

    /// Render the picker layout to a string for testing purposes.
    /// Note: ASCII only — does not support Unicode beyond single-byte chars.
    fn render_to_string(layout: UiTree, width: usize, height: usize) -> String {
        let mut grid = vec![' '; height * width];

        for item in layout.iter() {
            let x0 = item.pos[0] as usize;
            let y0 = item.pos[1] as usize;
            let item_width = item.size[0] as usize;
            let text = &item.data.0;

            for (i, c) in text.chars().take(item_width).enumerate() {
                if y0 < height && x0 + i < width {
                    grid[y0 * width + (x0 + i)] = c;
                }
            }
        }

        grid.chunks(width)
            .map(|row| row.iter().collect::<String>().trim_end().to_string())
            .join("\n")
    }

    #[test]
    fn test_picker_empty_input() {
        let items = create_test_items();
        let state = PickerState::new("Select branch", items, false);
        let config = test_config();

        let mut layout = LayoutTree::new();
        layout.vertical(None, crate::ui::layout::OPTS, |layout| {
            layout_picker(layout, &state, &config, 40);
        });
        layout.compute([40, 15]);

        insta::assert_snapshot!(render_to_string(layout, 40, 15));
    }

    #[test]
    fn test_picker_with_filter() {
        let items = create_test_items();
        let mut state = PickerState::new("Select branch", items, false);

        // Type "fea" to filter
        state
            .input_state
            .handle_key_event(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::empty()));
        state
            .input_state
            .handle_key_event(KeyEvent::new(KeyCode::Char('e'), KeyModifiers::empty()));
        state
            .input_state
            .handle_key_event(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty()));
        state.update_filter();

        let config = test_config();

        let mut layout = LayoutTree::new();
        layout.vertical(None, crate::ui::layout::OPTS, |layout| {
            layout_picker(layout, &state, &config, 40);
        });
        layout.compute([40, 15]);

        insta::assert_snapshot!(render_to_string(layout, 40, 15));
    }

    #[test]
    fn test_picker_cursor_movement() {
        let items = create_test_items();
        let mut state = PickerState::new("Select branch", items, false);

        // Move cursor to third item
        state.next();
        state.next();

        let config = test_config();

        let mut layout = LayoutTree::new();
        layout.vertical(None, crate::ui::layout::OPTS, |layout| {
            layout_picker(layout, &state, &config, 40);
        });
        layout.compute([40, 15]);

        insta::assert_snapshot!(render_to_string(layout, 40, 15));
    }

    #[test]
    fn test_picker_scroll_many_items() {
        // Create 20 items to test scrolling
        let items: Vec<_> = (0..20)
            .map(|i| {
                PickerItem::new(
                    format!("branch-{:02}", i),
                    PickerData::Revision(format!("branch-{:02}", i)),
                )
            })
            .collect();

        let mut state = PickerState::new("Select branch", items, false);
        let config = test_config();

        // Move cursor to middle (position 10)
        for _ in 0..10 {
            state.next();
        }

        let mut layout = LayoutTree::new();
        layout.vertical(None, crate::ui::layout::OPTS, |layout| {
            layout_picker(layout, &state, &config, 40);
        });
        layout.compute([40, 15]);

        insta::assert_snapshot!(render_to_string(layout, 40, 15));
    }

    #[test]
    fn test_picker_scroll_near_end() {
        // Create 20 items to test scrolling near the end
        let items: Vec<_> = (0..20)
            .map(|i| {
                PickerItem::new(
                    format!("branch-{:02}", i),
                    PickerData::Revision(format!("branch-{:02}", i)),
                )
            })
            .collect();

        let mut state = PickerState::new("Select branch", items, false);
        let config = test_config();

        // Move cursor near end (position 18)
        for _ in 0..18 {
            state.next();
        }

        let mut layout = LayoutTree::new();
        layout.vertical(None, crate::ui::layout::OPTS, |layout| {
            layout_picker(layout, &state, &config, 40);
        });
        layout.compute([40, 15]);

        insta::assert_snapshot!(render_to_string(layout, 40, 15));
    }

    #[test]
    fn test_picker_filtered_with_navigation() {
        let items = create_test_items();
        let mut state = PickerState::new("Select branch", items, false);

        // Filter to get feature branches
        state
            .input_state
            .handle_key_event(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::empty()));
        state
            .input_state
            .handle_key_event(KeyEvent::new(KeyCode::Char('e'), KeyModifiers::empty()));
        state
            .input_state
            .handle_key_event(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty()));
        state.update_filter();

        // Navigate to second item
        state.next();

        let config = test_config();

        let mut layout = LayoutTree::new();
        layout.vertical(None, crate::ui::layout::OPTS, |layout| {
            layout_picker(layout, &state, &config, 40);
        });
        layout.compute([40, 15]);

        insta::assert_snapshot!(render_to_string(layout, 40, 15));
    }

    #[test]
    fn test_picker_with_custom_input() {
        let items = create_test_items();
        let mut state = PickerState::new("New branch", items, true);

        // Type a custom branch name
        state
            .input_state
            .handle_key_event(KeyEvent::new(KeyCode::Char('m'), KeyModifiers::empty()));
        state
            .input_state
            .handle_key_event(KeyEvent::new(KeyCode::Char('y'), KeyModifiers::empty()));
        state.update_filter();

        let config = test_config();

        let mut layout = LayoutTree::new();
        layout.vertical(None, crate::ui::layout::OPTS, |layout| {
            layout_picker(layout, &state, &config, 40);
        });
        layout.compute([40, 15]);

        insta::assert_snapshot!(render_to_string(layout, 40, 15));
    }

    #[test]
    fn test_picker_no_matches() {
        let items = create_test_items();
        let mut state = PickerState::new("Select branch", items, false);

        // Type something that doesn't match
        state
            .input_state
            .handle_key_event(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::empty()));
        state
            .input_state
            .handle_key_event(KeyEvent::new(KeyCode::Char('y'), KeyModifiers::empty()));
        state
            .input_state
            .handle_key_event(KeyEvent::new(KeyCode::Char('z'), KeyModifiers::empty()));
        state.update_filter();

        let config = test_config();

        let mut layout = LayoutTree::new();
        layout.vertical(None, crate::ui::layout::OPTS, |layout| {
            layout_picker(layout, &state, &config, 40);
        });
        layout.compute([40, 15]);

        insta::assert_snapshot!(render_to_string(layout, 40, 15));
    }

    #[test]
    fn test_picker_narrow_width() {
        let items = create_test_items();
        let state = PickerState::new("Select", items, false);
        let config = test_config();

        let mut layout = LayoutTree::new();
        layout.vertical(None, crate::ui::layout::OPTS, |layout| {
            layout_picker(layout, &state, &config, 20);
        });
        layout.compute([20, 15]);

        insta::assert_snapshot!(render_to_string(layout, 20, 15));
    }
}
