use ratatui::prelude::*;

use crate::{config::Config, items::TargetData, Res};

use super::Item;
use std::{borrow::Cow, collections::HashSet, rc::Rc};

pub(crate) mod log;
pub(crate) mod show;
pub(crate) mod show_refs;
pub(crate) mod status;

const BOTTOM_CONTEXT_LINES: usize = 2;

pub(crate) struct Screen {
    pub(crate) cursor: usize,
    pub(crate) scroll: usize,
    pub(crate) size: Rect,
    config: Rc<Config>,
    refresh_items: Box<dyn Fn() -> Res<Vec<Item>>>,
    items: Vec<Item>,
    line_index: Vec<usize>,
    collapsed: HashSet<Cow<'static, str>>,
}

impl Screen {
    pub(crate) fn new(
        config: Rc<Config>,
        size: Rect,
        refresh_items: Box<dyn Fn() -> Res<Vec<Item>>>,
    ) -> Res<Self> {
        let mut screen = Self {
            cursor: 0,
            scroll: 0,
            size,
            config,
            refresh_items,
            items: vec![],
            line_index: vec![],
            collapsed: HashSet::new(),
        };

        screen.update()?;

        // TODO Maybe this should be done on update. Better keep track of toggled sections rather than collapsed then.
        screen
            .items
            .iter()
            .filter(|item| item.default_collapsed)
            .for_each(|item| {
                screen.collapsed.insert(item.id.clone());
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
                && matches!(self.at_line(line_i).target_data, Some(TargetData::Hunk(_)))
        })
    }

    fn find_first_selectable(&mut self) -> Option<usize> {
        (0..self.line_index.len()).find(|&line_i| !self.at_line(line_i).unselectable)
    }

    fn at_line(&mut self, line_i: usize) -> &Item {
        &self.items[self.line_index[line_i]]
    }

    pub(crate) fn select_next(&mut self) {
        self.cursor = self.find_next();
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

    pub(crate) fn find_next(&mut self) -> usize {
        (self.cursor..self.line_index.len())
            .skip(1)
            .find(|&line_i| !self.at_line(line_i).unselectable)
            .unwrap_or(self.cursor)
    }

    pub(crate) fn select_previous(&mut self) {
        self.cursor = (0..self.cursor)
            .rev()
            .find(|&line_i| !self.at_line(line_i).unselectable)
            .unwrap_or(self.cursor);

        self.scroll_fit_start();
    }

    pub(crate) fn scroll_half_page_up(&mut self) {
        let half_screen = self.size.height as usize / 2;
        self.scroll = self.scroll.saturating_sub(half_screen);
    }

    pub(crate) fn scroll_half_page_down(&mut self) {
        let half_screen = self.size.height as usize / 2;
        self.scroll = (self.scroll + half_screen).min(
            self.line_index
                .iter()
                .copied()
                .enumerate()
                .map(|(line, _)| (line + 1).saturating_sub(half_screen))
                .last()
                .unwrap_or(0),
        );
    }

    pub(crate) fn toggle_section(&mut self) {
        let selected = &self.items[self.line_index[self.cursor]];

        if selected.section {
            if self.collapsed.contains(&selected.id) {
                self.collapsed.remove(&selected.id);
            } else {
                self.collapsed.insert(selected.id.clone());
            }
        }

        self.update_line_index();
    }

    pub(crate) fn update(&mut self) -> Res<()> {
        self.items = (self.refresh_items)()?;
        self.update_line_index();
        self.clamp_cursor();
        self.move_from_unselectable();
        Ok(())
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

    fn clamp_cursor(&mut self) {
        self.cursor = self
            .cursor
            .clamp(0, self.line_index.len().saturating_sub(1));
    }

    fn move_from_unselectable(&mut self) {
        if self.get_selected_item().unselectable {
            self.select_previous();
        }
        if self.get_selected_item().unselectable {
            self.select_next();
        }
    }

    fn is_collapsed(&self, item: &Item) -> bool {
        self.collapsed.contains(&item.id)
    }

    pub(crate) fn get_selected_item(&self) -> &Item {
        &self.items[self.line_index[self.cursor]]
    }
}

impl Widget for &Screen {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = &self.config.style;

        let scan_start = self.scroll.min(self.cursor);
        let scan_end = (self.scroll + area.height as usize).min(self.line_index.len());
        let scan_highlight_range = scan_start..(scan_end);
        let context_lines = self.scroll - scan_start;

        for (line_i, (item_i, item, line, highlight_depth)) in self.line_index[scan_highlight_range]
            .iter()
            .copied()
            .scan(None, |highlight_depth, item_i| {
                let item = &self.items[item_i];
                if self.line_index[self.cursor] == item_i {
                    *highlight_depth = Some(item.depth);
                } else if highlight_depth.is_some_and(|s| s >= item.depth) {
                    *highlight_depth = None;
                };

                Some((item_i, item, &item.display, *highlight_depth))
            })
            .skip(context_lines)
            .enumerate()
        {
            let line_area = Rect {
                x: 0,
                y: line_i as u16,
                width: buf.area.width,
                height: 1,
            };

            let indented_line_area = Rect { x: 1, ..line_area };

            if highlight_depth.is_some() {
                if self.line_index[self.cursor] == item_i {
                    buf.set_style(line_area, &style.selection_line);
                } else {
                    buf.get_mut(0, line_i as u16)
                        .set_char('▌')
                        .set_style(&style.selection_bar);

                    buf.set_style(line_area, &style.selection_area);
                }
            }

            line.render(indented_line_area, buf);

            let mut occupied_right = 0;

            if self.is_collapsed(item) && line.width() > 0 {
                let pos = (indented_line_area.x + line.width() as u16 + 1).min(area.width - 1);
                buf.get_mut(pos, line_i as u16)
                    .set_char('⏷')
                    .set_style(Style::reset());

                occupied_right += 1;
            }

            if line.width() > line_area.width as usize - 2 {
                buf.get_mut(line_area.width - occupied_right - 1, line_i as u16)
                    .set_char('…');
            }

            if self.line_index[self.cursor] == item_i {
                buf.get_mut(0, line_i as u16).set_char('🢒');
            }
        }
    }
}
