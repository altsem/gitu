use crate::config::StyleConfig;
use crate::ui::layout::OPTS;
use crate::ui::{UiTree, layout_span};
use crate::{item_data::ItemData, ui};
use ratatui::{layout::Size, style::Style, text::Line};
use unicode_segmentation::UnicodeSegmentation;

use crate::{Res, config::Config, items::hash};

use super::Item;
use std::borrow::Cow;
use std::collections::HashSet;
use std::sync::Arc;

pub(crate) mod log;
pub(crate) mod show;
pub(crate) mod show_refs;
pub(crate) mod show_stash;
pub(crate) mod status;

const BOTTOM_CONTEXT_LINES: usize = 2;

#[derive(Copy, Clone, Debug)]
pub(crate) enum NavMode {
    Normal,
    Siblings { depth: usize },
    IncludeHunkLines,
}

enum SearchState {
    Inactive,
    Incremental {
        pattern: String,
        matches: Vec<usize>,
        current_match_index: Option<usize>,
        previous_cursor: usize,
        previous_scroll: usize,
        previous_collapsed: HashSet<u64>,
    },
    Active {
        pattern: String,
        matches: Vec<usize>,
        current_match_index: Option<usize>,
    },
}

pub(crate) struct Screen {
    pub(crate) size: Size,
    cursor: usize,
    scroll: usize,
    config: Arc<Config>,
    refresh_items: Box<dyn Fn() -> Res<Vec<Item>>>,
    items: Vec<Item>,
    line_index: Vec<usize>,
    collapsed: HashSet<u64>,
    search: SearchState,
}

impl Screen {
    pub(crate) fn new(
        config: Arc<Config>,
        size: Size,
        refresh_items: Box<dyn Fn() -> Res<Vec<Item>>>,
    ) -> Res<Self> {
        let collapsed = config
            .general
            .collapsed_sections
            .clone()
            .into_iter()
            .map(hash)
            .collect();

        let mut screen = Self {
            cursor: 0,
            scroll: 0,
            size,
            config,
            refresh_items,
            items: vec![],
            line_index: vec![],
            collapsed,
            search: SearchState::Inactive,
        };

        screen.update()?;

        // TODO Maybe this should be done on update. Better keep track of toggled sections rather than collapsed then.
        screen
            .items
            .iter()
            .filter(|item| item.default_collapsed)
            .for_each(|item| {
                screen.collapsed.insert(item.id);
            });
        screen.update_line_index();

        screen.cursor = screen
            .find_first_hunk()
            .or_else(|| screen.find_first_selectable())
            .unwrap_or(0);

        Ok(screen)
    }

    fn find_first_hunk(&mut self) -> Option<usize> {
        (0..self.line_index.len()).find(|&line_i| {
            !self.at_line(line_i).unselectable
                && matches!(self.at_line(line_i).data, ItemData::Hunk { .. })
        })
    }

    fn find_first_selectable(&mut self) -> Option<usize> {
        (0..self.line_index.len()).find(|&line_i| !self.at_line(line_i).unselectable)
    }

    fn at_line(&self, line_i: usize) -> &Item {
        &self.items[self.line_index[line_i]]
    }

    pub(crate) fn select_next(&mut self, nav_mode: NavMode) {
        self.cursor = self.find_next(nav_mode);
        self.scroll_fit_end();
        self.scroll_fit_start();
    }

    fn scroll_fit_start(&mut self) {
        if self.items.is_empty() {
            return;
        }

        let top = self.cursor.saturating_sub(self.get_selected_item().depth);
        if top < self.scroll {
            self.scroll = top;
        }
    }

    fn scroll_fit_end(&mut self) {
        if self.items.is_empty() {
            return;
        }

        let depth = self.get_selected_item().depth;

        let last = BOTTOM_CONTEXT_LINES
            + (self.cursor..self.line_index.len())
                .take_while(|&line_i| line_i == self.cursor || depth < self.at_line(line_i).depth)
                .last()
                .unwrap();

        let end_line = self.size.height.saturating_sub(1) as usize;
        if last > end_line + self.scroll {
            self.scroll = last - end_line;
        }
    }

    pub(crate) fn find_next(&mut self, nav_mode: NavMode) -> usize {
        (self.cursor..self.line_index.len())
            .skip(1)
            .find(|&line_i| self.nav_filter(line_i, nav_mode))
            .unwrap_or(self.cursor)
    }

    fn nav_filter(&self, line_i: usize, nav_mode: NavMode) -> bool {
        let item = self.at_line(line_i);
        match nav_mode {
            NavMode::Normal => {
                let is_hunk_line = matches!(item.data, ItemData::HunkLine { .. });

                !item.unselectable && !is_hunk_line
            }
            NavMode::Siblings { depth } => {
                !item.unselectable && item.data.is_section() && item.depth <= depth
            }
            NavMode::IncludeHunkLines => !item.unselectable,
        }
    }

    pub(crate) fn select_previous(&mut self, nav_mode: NavMode) {
        self.cursor = self.find_previous(nav_mode);
        self.scroll_fit_start();
    }

    fn find_previous(&mut self, nav_mode: NavMode) -> usize {
        (0..self.cursor)
            .rev()
            .find(|&line_i| self.nav_filter(line_i, nav_mode))
            .unwrap_or(self.cursor)
    }

    pub(crate) fn scroll_half_page_up(&mut self) {
        let half_screen = self.size.height as usize / 2;
        self.scroll = self.scroll.saturating_sub(half_screen);

        let nav_mode = self.selected_item_nav_mode();
        self.update_cursor(nav_mode);
    }

    pub(crate) fn scroll_half_page_down(&mut self) {
        let half_screen = self.size.height as usize / 2;
        self.scroll = (self.scroll + half_screen).min(
            self.line_index
                .iter()
                .copied()
                .enumerate()
                .map(|(line, _)| (line + 1).saturating_sub(half_screen))
                .next_back()
                .unwrap_or(0),
        );

        let nav_mode = self.selected_item_nav_mode();
        self.update_cursor(nav_mode);
    }

    pub(crate) fn scroll_up(&mut self, lines: usize) {
        self.scroll = self.scroll.saturating_sub(lines);
        let nav_mode = self.selected_item_nav_mode();
        self.update_cursor(nav_mode);
    }

    pub(crate) fn scroll_down(&mut self, lines: usize) {
        let max_scroll = self
            .line_index
            .len()
            .saturating_sub(self.size.height as usize);
        self.scroll = (self.scroll + lines).min(max_scroll);
        let nav_mode = self.selected_item_nav_mode();
        self.update_cursor(nav_mode);
    }

    pub(crate) fn toggle_section(&mut self) {
        let selected = &self.items[self.line_index[self.cursor]];

        if selected.data.is_section() {
            if self.collapsed.contains(&selected.id) {
                self.collapsed.remove(&selected.id);
            } else {
                self.collapsed.insert(selected.id);
            }
        }

        self.update_line_index();
    }

    pub(crate) fn update(&mut self) -> Res<()> {
        let nav_mode = self.selected_item_nav_mode();
        self.items = (self.refresh_items)()?;
        self.update_line_index();
        self.update_cursor(nav_mode);
        Ok(())
    }

    fn update_cursor(&mut self, nav_mode: NavMode) {
        self.clamp_cursor();
        if self.is_cursor_off_screen() {
            self.move_cursor_to_screen_center();
        }

        self.clamp_cursor();
        self.move_from_unselectable(nav_mode);
    }

    fn selected_item_nav_mode(&mut self) -> NavMode {
        if self.items.is_empty() {
            return NavMode::Normal;
        }

        match self.get_selected_item().data {
            ItemData::HunkLine { .. } => NavMode::IncludeHunkLines,
            _ => NavMode::Normal,
        }
    }

    fn update_line_index(&mut self) {
        self.line_index = self
            .items
            .iter()
            .enumerate()
            .scan(None, |collapse_depth, (i, next)| {
                if collapse_depth.is_some_and(|depth| depth < next.depth) {
                    return Some(None);
                }

                *collapse_depth = if next.data.is_section() && self.is_collapsed(next) {
                    Some(next.depth)
                } else {
                    None
                };

                Some(Some((i, next)))
            })
            .flatten()
            .map(|(i, _item)| i)
            .collect();

        // Ensure cursor and scroll are within bounds after line_index changes
        if !self.line_index.is_empty() {
            self.cursor = self.cursor.min(self.line_index.len() - 1);
            self.scroll = self.scroll.min(self.line_index.len() - 1);
        } else {
            self.cursor = 0;
            self.scroll = 0;
        }
    }

    fn is_cursor_off_screen(&self) -> bool {
        !self.line_views(self.size).any(|line| line.highlighted)
    }

    fn move_cursor_to_screen_center(&mut self) {
        let half_screen = self.size.height as usize / 2;
        self.cursor = self.scroll + half_screen;
    }

    fn clamp_cursor(&mut self) {
        self.cursor = self
            .cursor
            .clamp(0, self.line_index.len().saturating_sub(1));
    }

    fn move_from_unselectable(&mut self, nav_mode: NavMode) {
        if !self.nav_filter(self.cursor, nav_mode) {
            self.select_previous(nav_mode);
        }
        if !self.nav_filter(self.cursor, nav_mode) {
            self.select_next(nav_mode);
        }
    }

    pub(crate) fn move_cursor_to_screen_line(&mut self, screen_line: usize) {
        if self.line_index.is_empty() {
            return;
        }

        let new_cursor = screen_line + self.scroll;
        if new_cursor >= self.line_index.len() || self.cursor == new_cursor {
            return;
        }

        let old_cursor = self.cursor;
        self.cursor = new_cursor;

        let nav_mode = self.selected_item_nav_mode();
        self.move_from_unselectable(nav_mode);

        if !self.nav_filter(self.cursor, nav_mode) {
            // There was no selectable item, put the cursor back.
            self.cursor = old_cursor;
        } else {
            // Use minimal scrolling to keep the cursor visible.
            self.scroll_fit_start();
        }
    }

    pub(crate) fn is_collapsed(&self, item: &Item) -> bool {
        self.collapsed.contains(&item.id)
    }

    pub(crate) fn get_selected_item(&self) -> &Item {
        &self.items[self.line_index[self.cursor]]
    }

    pub(crate) fn is_search_match(&self, item_index: usize) -> bool {
        match &self.search {
            SearchState::Inactive => false,
            SearchState::Incremental { matches, .. } | SearchState::Active { matches, .. } => {
                matches.contains(&item_index)
            }
        }
    }

    pub(crate) fn is_current_search_match(&self, item_index: usize) -> bool {
        match &self.search {
            SearchState::Inactive => false,
            SearchState::Incremental {
                matches,
                current_match_index,
                ..
            }
            | SearchState::Active {
                matches,
                current_match_index,
                ..
            } => {
                if let Some(current_idx) = current_match_index {
                    matches.get(*current_idx) == Some(&item_index)
                } else {
                    false
                }
            }
        }
    }

    pub(crate) fn search(&mut self, pattern: &str, is_preview: bool) {
        if pattern.is_empty() {
            self.clear_search();
            return;
        }

        // Determine if pattern changed and get previous state if needed
        let (pattern_changed, previous_state) = match &self.search {
            SearchState::Inactive => (true, None),
            SearchState::Incremental {
                pattern: old_pattern,
                previous_collapsed,
                previous_cursor,
                previous_scroll,
                ..
            } => {
                let changed = old_pattern != pattern;
                if changed {
                    (
                        true,
                        Some((
                            previous_collapsed.clone(),
                            *previous_cursor,
                            *previous_scroll,
                        )),
                    )
                } else {
                    (
                        false,
                        Some((
                            previous_collapsed.clone(),
                            *previous_cursor,
                            *previous_scroll,
                        )),
                    )
                }
            }
            SearchState::Active {
                pattern: old_pattern,
                ..
            } => (old_pattern != pattern, None),
        };

        // Save the current collapsed state and cursor position before expanding for search (only on first preview)
        let (previous_collapsed, previous_cursor, previous_scroll) =
            if pattern_changed && previous_state.is_none() && is_preview {
                (self.collapsed.clone(), self.cursor, self.scroll)
            } else {
                previous_state.unwrap_or_else(|| (self.collapsed.clone(), self.cursor, self.scroll))
            };

        // Search through all items (not just visible ones)
        let pattern_lower = pattern.to_lowercase();
        let mut matches = Vec::new();
        for (item_index, item) in self.items.iter().enumerate() {
            let line_text = item
                .to_line(Arc::clone(&self.config))
                .to_string()
                .to_lowercase();
            if line_text.contains(&pattern_lower) {
                matches.push(item_index);
            }
        }

        // If pattern changed during preview, restore the previous collapsed state before re-expanding
        if pattern_changed && is_preview {
            self.collapsed = previous_collapsed.clone();
            self.update_line_index();
        }

        // Expand collapsed sections that contain matches
        for &item_index in &matches {
            self.expand_to_item(item_index);
        }

        // Update line index after expanding sections
        self.update_line_index();

        // Move to the first match (always forward)
        let current_match_index = if !matches.is_empty() {
            let current_item_index = self.line_index.get(self.cursor).copied().unwrap_or(0);

            // Find the first match at or after the current position
            let idx = matches
                .iter()
                .position(|&match_idx| match_idx >= current_item_index)
                .or(Some(0)); // If no match after cursor, wrap to first match

            if let Some(match_idx) = idx {
                self.move_to_match_with_matches(&matches, match_idx);
            }
            idx
        } else {
            // No matches found during preview - restore cursor and scroll position
            if is_preview {
                self.cursor = previous_cursor;
                self.scroll = previous_scroll;
            }
            None
        };

        // Update search state based on preview mode
        self.search = if is_preview {
            SearchState::Incremental {
                pattern: pattern.to_string(),
                matches,
                current_match_index,
                previous_collapsed,
                previous_cursor,
                previous_scroll,
            }
        } else {
            SearchState::Active {
                pattern: pattern.to_string(),
                matches,
                current_match_index,
            }
        };
    }

    pub(crate) fn search_next(&mut self) {
        let (matches, current_match_index) = match &self.search {
            SearchState::Inactive => return,
            SearchState::Incremental {
                matches,
                current_match_index,
                ..
            }
            | SearchState::Active {
                matches,
                current_match_index,
                ..
            } => {
                if matches.is_empty() {
                    return;
                }
                (matches.clone(), *current_match_index)
            }
        };

        // Move to next match (forward)
        let new_match_index = match current_match_index {
            Some(idx) if idx + 1 < matches.len() => Some(idx + 1),
            _ => Some(0), // Wrap to first match
        };

        if let Some(match_idx) = new_match_index {
            self.move_to_match_with_matches(&matches, match_idx);
        }

        // Update the search state with new current_match_index
        match &mut self.search {
            SearchState::Incremental {
                current_match_index,
                ..
            }
            | SearchState::Active {
                current_match_index,
                ..
            } => {
                *current_match_index = new_match_index;
            }
            SearchState::Inactive => {}
        }
    }

    pub(crate) fn search_previous(&mut self) {
        let (matches, current_match_index) = match &self.search {
            SearchState::Inactive => return,
            SearchState::Incremental {
                matches,
                current_match_index,
                ..
            }
            | SearchState::Active {
                matches,
                current_match_index,
                ..
            } => {
                if matches.is_empty() {
                    return;
                }
                (matches.clone(), *current_match_index)
            }
        };

        // Move to previous match (backward)
        let new_match_index = match current_match_index {
            Some(0) | None => Some(matches.len() - 1), // Wrap to last match
            Some(idx) => Some(idx - 1),
        };

        if let Some(match_idx) = new_match_index {
            self.move_to_match_with_matches(&matches, match_idx);
        }

        // Update the search state with new current_match_index
        match &mut self.search {
            SearchState::Incremental {
                current_match_index,
                ..
            }
            | SearchState::Active {
                current_match_index,
                ..
            } => {
                *current_match_index = new_match_index;
            }
            SearchState::Inactive => {}
        }
    }

    pub(crate) fn clear_search(&mut self) {
        // Restore the previous state if in incremental search
        let restore_state = if let SearchState::Incremental {
            previous_collapsed,
            previous_cursor,
            previous_scroll,
            ..
        } = &self.search
        {
            Some((previous_collapsed.clone(), *previous_cursor, *previous_scroll))
        } else {
            None
        };

        if let Some((previous_collapsed, previous_cursor, previous_scroll)) = restore_state {
            self.collapsed = previous_collapsed;
            self.update_line_index();
            self.cursor = previous_cursor;
            self.scroll = previous_scroll;
        }

        self.search = SearchState::Inactive;
    }

    pub(crate) fn get_search_pattern(&self) -> Option<&str> {
        match &self.search {
            SearchState::Inactive => None,
            SearchState::Incremental { pattern, .. } | SearchState::Active { pattern, .. } => {
                Some(pattern.as_str())
            }
        }
    }

    fn expand_to_item(&mut self, item_index: usize) {
        // Expand all parent sections to make this item visible
        let item = &self.items[item_index];
        let target_depth = item.depth;

        // Find all sections that could be parents of this item
        for i in (0..item_index).rev() {
            let potential_parent = &self.items[i];
            if potential_parent.data.is_section() && potential_parent.depth < target_depth {
                // This is a parent section, expand it
                self.collapsed.remove(&potential_parent.id);
            }
        }
    }

    fn move_to_match_with_matches(&mut self, matches: &[usize], match_index: usize) {
        if let Some(&item_index) = matches.get(match_index) {
            // Find the line_index that corresponds to this item_index
            if let Some(line_i) = self.line_index.iter().position(|&idx| idx == item_index) {
                self.cursor = line_i;
                self.scroll_fit_end();
                self.scroll_fit_start();
            }
        }
    }

    pub(crate) fn is_valid_screen_line(&self, screen_line: usize) -> bool {
        let target_line_i = screen_line + self.scroll;
        if self.line_index.is_empty() || target_line_i >= self.line_index.len() {
            return false;
        }
        self.nav_filter(target_line_i, NavMode::IncludeHunkLines)
    }

    fn line_views(&'_ self, area: Size) -> impl Iterator<Item = LineView<'_>> {
        let scan_start = self.scroll.min(self.cursor).min(self.line_index.len());
        let scan_end = (self.scroll + area.height as usize).min(self.line_index.len());
        // Ensure scan_end is never less than scan_start
        let scan_end = scan_end.max(scan_start);
        let scan_highlight_range = scan_start..scan_end;
        let context_lines = self.scroll.saturating_sub(scan_start);

        self.line_index[scan_highlight_range]
            .iter()
            .scan(None, |highlight_depth, item_index| {
                let item = &self.items[*item_index];
                if self.line_index[self.cursor] == *item_index {
                    *highlight_depth = Some(item.depth);
                } else if highlight_depth.is_some_and(|s| s >= item.depth) {
                    *highlight_depth = None;
                };
                let display = item.to_line(Arc::clone(&self.config));

                Some(LineView {
                    item_index: *item_index,
                    display,
                    highlighted: highlight_depth.is_some(),
                })
            })
            .skip(context_lines)
    }
}

struct LineView<'a> {
    item_index: usize,
    display: Line<'a>,
    highlighted: bool,
}

const SPACES: &str = "                                                                ";

pub(crate) fn layout_screen<'a>(
    layout: &mut UiTree<'a>,
    size: Size,
    screen: &'a Screen,
    hide_cursor: bool,
) {
    let style = &screen.config.style;

    layout.vertical(None, OPTS, |layout| {
        for line in screen.line_views(size) {
            layout.horizontal(None, OPTS, |layout| {
                let is_line_sel = screen.line_index[screen.cursor] == line.item_index;
                let area_sel = area_selection_highlight(style, &line);
                let line_sel = line_selection_highlight(style, &line, is_line_sel);
                let bg = area_sel.patch(line_sel);

                let mut line_end = 1;
                let gutter_char = if !hide_cursor && line.highlighted {
                    gutter_char(style, is_line_sel, bg)
                } else {
                    (" ".into(), Style::new())
                };

                layout_span(layout, gutter_char);

                line.display.spans.into_iter().for_each(|span| {
                    let span_width = span.content.graphemes(true).count();

                    // Check if we need to highlight search matches in this span
                    if let Some(pattern) = screen.get_search_pattern() {
                        if screen.is_search_match(line.item_index) && !pattern.is_empty() {
                            // Split the span into parts: non-match and match
                            let pattern_lower = pattern.to_lowercase();
                            let content_lower = span.content.to_lowercase();

                            let mut pos = 0;
                            let content_chars: Vec<char> = span.content.chars().collect();

                            for (match_start, _) in content_lower.match_indices(&pattern_lower) {
                                // Add non-match part before this match
                                if match_start > pos {
                                    let before: String =
                                        content_chars[pos..match_start].iter().collect();
                                    let before_style =
                                        bg.patch(line.display.style).patch(span.style);
                                    ui::layout_span(layout, (before.into(), before_style));
                                }

                                // Add highlighted match part
                                let match_end = match_start + pattern.chars().count();
                                let matched: String =
                                    content_chars[match_start..match_end].iter().collect();

                                // Use current_search_match style for the current match, search_match for others
                                let search_style =
                                    if screen.is_current_search_match(line.item_index) {
                                        &style.current_search_match
                                    } else {
                                        &style.search_match
                                    };

                                let match_style = bg
                                    .patch(line.display.style)
                                    .patch(span.style)
                                    .patch(Style::from(search_style));
                                ui::layout_span(layout, (matched.into(), match_style));

                                pos = match_end;
                            }

                            // Add remaining non-match part
                            if pos < content_chars.len() {
                                let after: String = content_chars[pos..].iter().collect();
                                let after_style = bg.patch(line.display.style).patch(span.style);
                                ui::layout_span(layout, (after.into(), after_style));
                            }

                            return;
                        }
                    }

                    // No search highlighting needed
                    let style = bg.patch(line.display.style).patch(span.style);

                    if line_end + span_width >= size.width as usize {
                        // Truncate the span and insert an ellipsis to indicate overflow
                        let overflow = line_end + span_width - size.width as usize;
                        line_end = size.width as usize;
                        ui::layout_span(
                            layout,
                            (
                                span.content
                                    .graphemes(true)
                                    .take(span_width.saturating_sub(overflow + 1))
                                    .collect::<String>()
                                    .into(),
                                style,
                            ),
                        );
                        layout_span(layout, ("…".into(), bg));
                    } else {
                        // Insert the span as normal
                        line_end += span_width;
                        ui::layout_span(layout, (span.content, style));
                    }
                });

                // Add ellipsis indicator for collapsed sections
                let item = &screen.items[line.item_index];
                if screen.is_collapsed(item) {
                    line_end += 1;
                    layout_span(layout, ("…".into(), bg));
                }

                // Style the rest of the line's empty space
                let style = if is_line_sel { line_sel } else { area_sel };
                let padding_width = (size.width as usize).saturating_sub(line_end);
                ui::repeat_chars(layout, padding_width, SPACES, style);
            });
        }
    });
}

fn gutter_char<'a>(style: &'a StyleConfig, is_line_sel: bool, bg: Style) -> (Cow<'a, str>, Style) {
    if is_line_sel {
        (
            style.cursor.symbol.to_string().into(),
            bg.patch(Style::from(&style.cursor)),
        )
    } else {
        (
            style.selection_bar.symbol.to_string().into(),
            bg.patch(Style::from(&style.selection_bar)),
        )
    }
}

fn line_selection_highlight(style: &StyleConfig, line: &LineView, selected_line: bool) -> Style {
    if line.highlighted && selected_line {
        Style::from(&style.selection_line)
    } else {
        Style::new()
    }
}

fn area_selection_highlight(style: &StyleConfig, line: &LineView) -> Style {
    if line.highlighted {
        Style::from(&style.selection_area)
    } else {
        Style::new()
    }
}
