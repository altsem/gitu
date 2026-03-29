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

pub(crate) struct Screen {
    pub(crate) size: Size,
    cursor: usize,
    scroll: usize,
    config: Arc<Config>,
    refresh_items: Box<dyn Fn() -> Res<Vec<Item>>>,
    items: Vec<Item>,
    line_index: Vec<usize>,
    collapsed: HashSet<u64>,
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

    pub(crate) fn is_valid_screen_line(&self, screen_line: usize) -> bool {
        let target_line_i = screen_line + self.scroll;
        if self.line_index.is_empty() || target_line_i >= self.line_index.len() {
            return false;
        }
        self.nav_filter(target_line_i, NavMode::IncludeHunkLines)
    }

    fn line_views(&'_ self, area: Size) -> impl Iterator<Item = LineView<'_>> {
        let scan_start = self.scroll.min(self.cursor);
        let scan_end = (self.scroll + area.height as usize).min(self.line_index.len());
        let scan_highlight_range = scan_start..(scan_end);
        let context_lines = self.scroll - scan_start;

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
                    let style = bg.patch(line.display.style).patch(span.style);

                    let span_width = span.content.graphemes(true).count();

                    line_end += span_width;
                    ui::layout_span(layout, (span.content, style));
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
