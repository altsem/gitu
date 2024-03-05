use ratatui::prelude::*;

use crate::{config::Config, items::TargetData, Res};

use super::Item;
use std::{borrow::Cow, collections::HashSet, rc::Rc};

pub(crate) mod log;
pub(crate) mod show;
pub(crate) mod show_refs;
pub(crate) mod status;

pub(crate) struct Screen {
    pub(crate) cursor: usize,
    pub(crate) scroll: u16,
    pub(crate) size: Rect,
    config: Rc<Config>,
    refresh_items: Box<dyn Fn() -> Res<Vec<Item>>>,
    items: Vec<Item>,
    collapsed: HashSet<Cow<'static, str>>,
}

impl<'a> Screen {
    pub(crate) fn new(
        config: Rc<Config>,
        size: Rect,
        refresh_items: Box<dyn Fn() -> Res<Vec<Item>>>,
    ) -> Res<Self> {
        let items = refresh_items()?;

        let mut collapsed = HashSet::new();
        items
            .iter()
            .filter(|item| item.default_collapsed)
            .for_each(|item| {
                collapsed.insert(item.id.clone());
            });

        let mut screen = Self {
            cursor: 0,
            scroll: 0,
            size,
            config,
            refresh_items,
            items,
            collapsed,
        };

        screen.cursor = screen
            .find_first_hunk()
            .or_else(|| screen.find_first_selectable())
            .unwrap_or(0);

        Ok(screen)
    }

    fn find_first_hunk(&mut self) -> Option<usize> {
        self.collapsed_items_iter()
            .find(|(_i, item)| {
                !item.unselectable && matches!(item.target_data, Some(TargetData::Hunk(_)))
            })
            .map(|(i, _item)| i)
    }

    fn find_first_selectable(&mut self) -> Option<usize> {
        self.collapsed_items_iter()
            .find(|(_i, item)| !item.unselectable)
            .map(|(i, _item)| i)
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

        let start_line = self
            .collapsed_items_iter()
            .find(|(i, item)| self.selected_or_direct_ancestor(item, i))
            .map(|(i, _item)| i)
            .unwrap() as u16;

        if start_line < self.scroll {
            self.scroll = start_line;
        }
    }

    fn selected_or_direct_ancestor(&self, item: &Item, i: &usize) -> bool {
        let levels_above = self.get_selected_item().depth.saturating_sub(item.depth);
        i == &(self.cursor - levels_above)
    }

    fn scroll_fit_end(&mut self) {
        if self.items.is_empty() {
            return;
        }

        let depth = self.items[self.cursor].depth;
        let last = 1 + self
            .collapsed_items_iter()
            .skip_while(|(i, _item)| i < &self.cursor)
            .take_while(|(i, item)| i == &self.cursor || depth < item.depth)
            .map(|(i, _item)| i + 1)
            .last()
            .unwrap();

        let end_line = self.size.height.saturating_sub(1);
        if last as u16 > end_line + self.scroll {
            self.scroll = last as u16 - end_line;
        }
    }

    pub(crate) fn find_next(&mut self) -> usize {
        self.collapsed_items_iter()
            .find(|(i, item)| i > &self.cursor && !item.unselectable)
            .map(|(i, _item)| i)
            .unwrap_or(self.cursor)
    }

    pub(crate) fn select_previous(&mut self) {
        self.cursor = self
            .collapsed_items_iter()
            .filter(|(i, item)| i < &self.cursor && !item.unselectable)
            .last()
            .map(|(i, _item)| i)
            .unwrap_or(self.cursor);

        self.scroll_fit_start();
    }

    pub(crate) fn scroll_half_page_up(&mut self) {
        let half_screen = self.size.height / 2;
        self.scroll = self.scroll.saturating_sub(half_screen);
    }

    pub(crate) fn scroll_half_page_down(&mut self) {
        let half_screen = self.size.height / 2;
        self.scroll = (self.scroll + half_screen).min(
            self.collapsed_items_iter()
                .map(|(i, _item)| (i + 1).saturating_sub(half_screen as usize))
                .last()
                .unwrap_or(0) as u16,
        );
    }

    pub(crate) fn toggle_section(&mut self) {
        let selected = &self.items[self.cursor];

        if selected.section {
            if self.collapsed.contains(&selected.id) {
                self.collapsed.remove(&selected.id);
            } else {
                self.collapsed.insert(selected.id.clone());
            }
        }
    }

    pub(crate) fn clamp_cursor(&mut self) {
        self.cursor = self.cursor.clamp(0, self.items.len().saturating_sub(1))
    }

    pub(crate) fn update(&mut self) -> Res<()> {
        self.items = (self.refresh_items)()?;
        Ok(())
    }

    pub(crate) fn collapsed_items_iter(&'a self) -> impl Iterator<Item = (usize, &Item)> {
        self.items
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
    }

    fn is_collapsed(&self, item: &Item) -> bool {
        self.collapsed.contains(&item.id)
    }

    pub(crate) fn get_selected_item(&self) -> &Item {
        &self.items[self.cursor]
    }
}

impl Widget for &Screen {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = &self.config.style;

        for (line_i, (item_i, item, line, highlight_depth)) in self
            .collapsed_items_iter()
            .scan(None, |highlight_depth, (item_i, item)| {
                if self.cursor == item_i {
                    *highlight_depth = Some(item.depth);
                } else if highlight_depth.is_some_and(|s| s >= item.depth) {
                    *highlight_depth = None;
                };

                Some((item_i, item, &item.display, *highlight_depth))
            })
            .skip(self.scroll as usize)
            .take(area.height as usize)
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
                if self.cursor == item_i {
                    buf.set_style(line_area, &style.selection_line);
                } else {
                    buf.get_mut(0, line_i as u16)
                        .set_char('▌')
                        .set_style(&style.selection_bar);

                    buf.set_style(line_area, &style.selection_area);
                }
            }

            line.render(indented_line_area, buf);
            let overflow = line.width() > line_area.width as usize;

            if self.is_collapsed(item) && line.width() > 0 || overflow {
                let line_end = (indented_line_area.x + line.width() as u16).min(area.width - 1);
                buf.get_mut(line_end, line_i as u16).set_char('…');
            }
            if self.cursor == item_i {
                buf.get_mut(0, line_i as u16).set_char('🢒');
            }
        }
    }
}
