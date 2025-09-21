use crate::item_data::ItemData;
use ratatui::{
    buffer::Buffer,
    layout::{Rect, Size},
    text::Line,
    widgets::Widget,
};

use crate::{config::Config, items::hash, Res};

use super::Item;
use std::{collections::HashSet, rc::Rc};

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
    config: Rc<Config>,
    refresh_items: Box<dyn Fn() -> Res<Vec<Item>>>,
    items: Vec<Item>,
    line_index: Vec<usize>,
    collapsed: HashSet<u64>,
}

impl Screen {
    pub(crate) fn new(
        config: Rc<Config>,
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

    fn at_line(&mut self, line_i: usize) -> &Item {
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

    fn nav_filter(&mut self, line_i: usize, nav_mode: NavMode) -> bool {
        let item = self.at_line(line_i);
        match nav_mode {
            NavMode::Normal => {
                let is_hunk_line = matches!(item.data, ItemData::HunkLine { .. });

                !item.unselectable && !is_hunk_line
            }
            NavMode::Siblings { depth } => {
                !item.unselectable && item.section && item.depth <= depth
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

        if selected.section {
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

                *collapse_depth = if next.section && self.is_collapsed(next) {
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

        // Build a mapping from screen lines to cursor indices by recreating
        // the same logic used in line_views.
        let area_height = self.size.height as usize;
        let scan_start = self.scroll.min(self.cursor);
        let scan_end = (self.scroll + area_height).min(self.line_index.len());
        let context_lines = self.scroll - scan_start;
        let displayed_line_index: Vec<usize> = (scan_start..scan_end).skip(context_lines).collect();
        if screen_line < displayed_line_index.len() {
            let new_cursor = displayed_line_index[screen_line];
            if self.cursor == new_cursor {
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
    }

    fn is_collapsed(&self, item: &Item) -> bool {
        self.collapsed.contains(&item.id)
    }

    pub(crate) fn get_selected_item(&self) -> &Item {
        &self.items[self.line_index[self.cursor]]
    }

    fn line_views(&self, area: Size) -> impl Iterator<Item = LineView<'_>> {
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
                let display = item.to_line(Rc::clone(&self.config));

                Some(LineView {
                    item_index: *item_index,
                    item,
                    display,
                    highlighted: highlight_depth.is_some(),
                })
            })
            .skip(context_lines)
    }
}

struct LineView<'a> {
    item_index: usize,
    item: &'a Item,
    display: Line<'a>,
    highlighted: bool,
}

impl Widget for &Screen {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = &self.config.style;

        for (line_index, line) in self.line_views(area.as_size()).enumerate() {
            let line_area = Rect {
                x: 0,
                y: line_index as u16,
                width: buf.area.width,
                height: 1,
            };

            let indented_line_area = Rect { x: 1, ..line_area };

            if line.highlighted {
                buf.set_style(line_area, &style.selection_area);

                if self.line_index[self.cursor] == line.item_index {
                    buf.set_style(line_area, &style.selection_line);
                } else {
                    buf[(0, line_index as u16)]
                        .set_char(style.selection_bar.symbol)
                        .set_style(&style.selection_bar);
                }
            }

            let line_width = line.display.width();

            line.display.render(indented_line_area, buf);
            let overflow = line_width > line_area.width as usize;

            let line_width = line_width as u16;

            if self.is_collapsed(line.item) && line_width > 0 || overflow {
                let line_end = (indented_line_area.x + line_width).min(area.width - 1);
                buf[(line_end, line_index as u16)].set_char('…');
            }

            if self.line_index[self.cursor] == line.item_index {
                buf[(0, line_index as u16)]
                    .set_char(style.cursor.symbol)
                    .set_style(&style.cursor);
            }
        }
    }
}
