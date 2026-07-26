use crate::config::StyleConfig;
use crate::ui::layout::{LayoutTree, OPTS};
use crate::ui::{UiItem, UiTree, layout_span};
use crate::{item_data::ItemData, ui};
use itertools::Itertools;
use ratatui::style::Style;

use crate::{Res, config::Config, items::hash};

use super::Item;
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::sync::Arc;

pub(crate) mod blame;
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
    IncludeSubLines,
}

pub(crate) struct Screen {
    pub(crate) size: (u16, u16),
    cursor: usize,
    scroll: usize,
    config: Arc<Config>,
    refresh_items: Box<dyn Fn() -> Res<Vec<Item>>>,
    items: Vec<Item>,
    /// Set of item (by their index in `items`) that are not collapsed
    expanded_items: BTreeSet<usize>,
    /// Maps line -> item, items spanning multiple lines appear as duplicates
    line_index: Vec<usize>,
    /// Maps line -> item, but only their first line
    unique_line_index: BTreeMap<usize, usize>,
    item_heights: Vec<u16>,
    collapsed: HashSet<u64>,
}

impl Screen {
    pub(crate) fn new(
        config: Arc<Config>,
        size: (u16, u16),
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
            expanded_items: BTreeSet::new(),
            line_index: vec![],
            unique_line_index: BTreeMap::new(),
            item_heights: vec![],
            collapsed,
        };

        screen.refresh()?;

        // TODO Maybe this should be done on update. Better keep track of toggled sections rather than collapsed then.
        screen
            .items
            .iter()
            .filter(|item| item.default_collapsed)
            .for_each(|item| {
                screen.collapsed.insert(item.id);
            });
        screen.update_indices()?;

        screen.cursor = screen
            .find_first_hunk()
            .or_else(|| screen.find_item(|item| !item.unselectable))
            .unwrap_or(0);

        Ok(screen)
    }

    fn find_first_hunk(&mut self) -> Option<usize> {
        self.find_item(|item| !item.unselectable && matches!(item.data, ItemData::Hunk { .. }))
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
        if self.line_index.is_empty() {
            return;
        }
        let Some(line_of_item) = self.line_of_item(self.cursor) else {
            return;
        };
        let top = line_of_item.saturating_sub(self.get_selected_item().depth);
        if top < self.scroll {
            self.scroll = top;
        }
    }

    fn scroll_fit_end(&mut self) {
        if self.line_index.is_empty() {
            return;
        }

        let depth = self.get_selected_item().depth;
        let current_item_i = self.cursor;

        let Some(line_of_item) = self.line_of_item(self.cursor) else {
            return;
        };

        let Some(last_item_line) = (line_of_item..self.line_index.len())
            .take_while(|&line_i| {
                self.line_index[line_i] == current_item_i || depth < self.at_line(line_i).depth
            })
            .last()
        else {
            return;
        };

        let last = BOTTOM_CONTEXT_LINES + last_item_line;

        let end_line = self.size.1.saturating_sub(1) as usize;
        if last > end_line + self.scroll {
            self.scroll = last - end_line;
        }
    }

    pub(crate) fn find_next(&mut self, nav_mode: NavMode) -> usize {
        (self.cursor..self.items.len())
            .skip(1)
            .find(|&item_i| self.nav_filter(item_i, nav_mode))
            .unwrap_or(self.cursor)
    }

    fn nav_filter(&self, item_i: usize, nav_mode: NavMode) -> bool {
        if !self.expanded_items.contains(&item_i) {
            return false;
        }

        let item = &self.items[item_i];
        match nav_mode {
            NavMode::Normal => {
                let is_sub_line = matches!(
                    item.data,
                    ItemData::HunkLine { .. } | ItemData::BlameCodeLine { .. }
                );
                !item.unselectable && !is_sub_line
            }
            NavMode::Siblings { depth } => {
                !item.unselectable && item.data.is_section() && item.depth <= depth
            }
            NavMode::IncludeSubLines => !item.unselectable,
        }
    }

    pub(crate) fn select_previous(&mut self, nav_mode: NavMode) {
        self.cursor = self.find_previous(nav_mode);
        self.scroll_fit_start();
    }

    fn find_previous(&mut self, nav_mode: NavMode) -> usize {
        (0..self.cursor)
            .rfind(|&item_i| self.nav_filter(item_i, nav_mode))
            .unwrap_or(self.cursor)
    }

    pub(crate) fn scroll_view_half_page_up(&mut self) {
        let half_screen = self.size.1 as usize / 2;
        self.scroll_view_up(half_screen);
    }

    pub(crate) fn scroll_view_half_page_down(&mut self) {
        let half_screen = self.size.1 as usize / 2;
        self.scroll_view_down(half_screen);
    }

    pub(crate) fn scroll_view_up(&mut self, lines: usize) {
        self.scroll = self.scroll.saturating_sub(lines);
        self.clamp_scroll();
    }

    pub(crate) fn scroll_view_down(&mut self, lines: usize) {
        self.scroll = self.scroll.saturating_add(lines);
        self.clamp_scroll();
    }

    pub(crate) fn toggle_section(&mut self) -> Res<()> {
        let selected = &self.items[self.cursor];

        if selected.data.is_section() {
            if self.collapsed.contains(&selected.id) {
                self.collapsed.remove(&selected.id);
            } else {
                self.collapsed.insert(selected.id);
            }
        }

        self.update_indices()?;
        Ok(())
    }

    pub(crate) fn refresh(&mut self) -> Res<()> {
        self.items = (self.refresh_items)()?;
        self.update_indices()?;
        self.update_cursor();
        Ok(())
    }

    pub(crate) fn resize(&mut self, w: u16, h: u16) -> Res<()> {
        self.size = (w, h);
        self.update_indices()?;
        self.update_cursor();
        Ok(())
    }

    fn update_cursor(&mut self) {
        // Nothing is selectable (e.g. the log of a branch with no commits).
        // Reset the cursor to a valid sentinel rather than positioning it,
        // which would index into an empty `line_index` and panic (#262).
        if self.line_index.is_empty() {
            self.cursor = 0;
            return;
        }

        self.clamp_scroll();
        self.clamp_cursor();
        if self.is_cursor_off_screen() {
            self.move_cursor_to_screen_center();
        }

        self.clamp_cursor();
        let nav_mode = self.selected_item_nav_mode();
        self.move_from_unselectable(nav_mode);
    }

    fn selected_item_nav_mode(&mut self) -> NavMode {
        if self.items.is_empty() {
            return NavMode::Normal;
        }

        match self.get_selected_item().data {
            ItemData::HunkLine { .. } | ItemData::BlameCodeLine { .. } => NavMode::IncludeSubLines,
            _ => NavMode::Normal,
        }
    }

    pub(crate) fn update_indices(&mut self) -> Res<()> {
        self.update_item_heights();

        assert_eq!(
            self.items.len(),
            self.item_heights.len(),
            "items and item_heights should have equal len"
        );

        self.line_index = self
            .filter_collapsed_items(&self.items)
            .flat_map(|(i, _item)| [i].repeat(self.item_heights[i] as usize))
            .collect();

        self.unique_line_index = self
            .line_index
            .iter()
            .cloned()
            .enumerate()
            .unique_by(|&(_, v)| v)
            .collect();

        self.expanded_items = self
            .filter_collapsed_items(&self.items)
            .map(|(i, _)| i)
            .collect();

        self.clamp_scroll();
        Ok(())
    }

    fn update_item_heights(&mut self) {
        self.item_heights = (0..self.items.len())
            .map(|item_index| {
                let mut layout = LayoutTree::new();
                let view = ItemView {
                    item_index,
                    highlighted: false,
                };
                layout_item(&mut layout, self, false, view);
                layout.compute([self.size.0, self.size.1]);

                layout
                    .iter()
                    .map(|item| item.pos[1] + item.size[1])
                    .max()
                    .unwrap_or(0)
            })
            .collect()
    }

    // FIXME Need to consider this when navigating
    fn filter_collapsed_items<'a>(
        &'a self,
        items: &'a [Item],
    ) -> impl Iterator<Item = (usize, &'a Item)> {
        items
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
    }

    fn is_cursor_off_screen(&self) -> bool {
        !self.item_views(self.size).any(|item| item.highlighted)
    }

    fn move_cursor_to_screen_center(&mut self) {
        let half_screen = self.size.1 as usize / 2;
        self.cursor = self.line_index[self.scroll + half_screen];
    }

    fn clamp_cursor(&mut self) {
        self.cursor = self.cursor.clamp(0, self.items.len().saturating_sub(1));
    }

    fn clamp_scroll(&mut self) {
        if self.line_index.is_empty() {
            self.scroll = 0;
            return;
        }

        self.scroll = self.scroll.min(self.max_scroll_with_context());
    }

    fn max_scroll_with_context(&self) -> usize {
        let len = self.line_index.len();
        if len == 0 {
            return 0;
        }

        let max_scroll = len.saturating_sub(self.size.1 as usize);
        let max_scroll = max_scroll.saturating_add(BOTTOM_CONTEXT_LINES);
        max_scroll.min(len.saturating_sub(1))
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
        let Some(&new_cursor) = self.line_index.get(screen_line + self.scroll) else {
            return;
        };
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

    pub(crate) fn move_cursor_to_top(&mut self) {
        if self.unique_line_index.is_empty() {
            return;
        }
        if let Some(first) = self.find_item(|item| !item.unselectable) {
            self.cursor = first;
            self.scroll = 0;
        }
    }

    pub(crate) fn move_cursor_to_bottom(&mut self) {
        if self.unique_line_index.is_empty() {
            return;
        }
        if let Some(last) = self.rfind_item(|item| !item.unselectable) {
            self.cursor = last;
            self.scroll_fit_end();
        }
    }

    pub(crate) fn is_collapsed(&self, item: &Item) -> bool {
        self.collapsed.contains(&item.id)
    }

    pub(crate) fn get_selected_item(&self) -> &Item {
        &self.items[self.cursor]
    }

    pub(crate) fn select_matching<F: Fn(&ItemData) -> bool>(&mut self, predicate: F) -> bool {
        if let Some(item_i) = self.find_item(|item| !item.unselectable && predicate(&item.data)) {
            self.cursor = item_i;
            let half_screen = self.size.1 as usize / 2;
            let Some(line_of_item) = self.line_of_item(self.cursor) else {
                return false;
            };

            if line_of_item >= half_screen {
                self.scroll = line_of_item - half_screen;
            }

            self.scroll_fit_end();
            self.scroll_fit_start();

            true
        } else {
            false
        }
    }

    pub(crate) fn select_last_matching<F: Fn(&ItemData) -> bool>(&mut self, predicate: F) -> bool {
        if let Some(item_i) = self.rfind_item(|item| !item.unselectable && predicate(&item.data)) {
            self.cursor = item_i;
            let half_screen = self.size.1 as usize / 2;
            let Some(line_of_item) = self.line_of_item(self.cursor) else {
                return false;
            };

            if line_of_item >= half_screen {
                self.scroll = line_of_item - half_screen;
            } else {
                self.scroll_fit_start();
            }

            true
        } else {
            false
        }
    }

    fn find_item<P: Fn(&Item) -> bool>(&self, predicate: P) -> Option<usize> {
        self.unique_line_index
            .iter()
            .find(|&(_, &item_i)| predicate(&self.items[item_i]))
            .map(|(_, &item_i)| item_i)
    }

    fn rfind_item<P: Fn(&Item) -> bool>(&self, predicate: P) -> Option<usize> {
        self.unique_line_index
            .iter()
            .rfind(|&(_, &item_i)| predicate(&self.items[item_i]))
            .map(|(_, &item_i)| item_i)
    }

    pub(crate) fn is_valid_screen_line(&self, screen_line: usize) -> bool {
        let Some(target_item_i) = self.line_of_item(screen_line + self.scroll) else {
            return false;
        };
        self.nav_filter(target_item_i, NavMode::IncludeSubLines)
    }

    fn line_of_item(&self, item_i: usize) -> Option<usize> {
        self.unique_line_index
            .iter()
            .find(|&(_, &i)| item_i == i)
            .map(|(&line_i, _)| line_i)
    }

    fn item_views(&'_ self, area: (u16, u16)) -> impl Iterator<Item = ItemView> {
        let first_visible_item = self
            .line_index
            .get(self.scroll)
            .cloned()
            .unwrap_or(self.items.len().saturating_sub(1));

        let scan_start_item = first_visible_item.min(self.cursor);
        let scan_end_line = (self.scroll + area.1 as usize).min(self.line_index.len());
        let scan_end_item = self
            .line_index
            .get(scan_end_line)
            .cloned()
            .unwrap_or(self.items.len());

        let scan_highlight_range = scan_start_item..(scan_end_item);
        let context_offset = self
            .expanded_items
            .range(scan_start_item..first_visible_item)
            .count();

        self.filter_collapsed_items(&self.items[scan_highlight_range])
            .scan(None, move |highlight_depth, (offset_item_index, item)| {
                let item_index = scan_start_item + offset_item_index;
                if self.cursor == item_index {
                    *highlight_depth = Some(item.depth);
                } else if highlight_depth.is_some_and(|s| s >= item.depth) {
                    *highlight_depth = None;
                };

                Some(ItemView {
                    item_index,
                    highlighted: highlight_depth.is_some(),
                })
            })
            .skip(context_offset)
    }
}

struct ItemView {
    item_index: usize,
    highlighted: bool,
}

pub(crate) fn layout_screen<'a>(layout: &mut UiTree<'a>, screen: &'a Screen, hide_cursor: bool) {
    layout.vertical(None, OPTS.fill_x(), |layout| {
        for view in screen.item_views(screen.size) {
            layout_item(layout, screen, hide_cursor, view);
        }
    });
}

fn layout_item<'a>(layout: &mut UiTree<'a>, screen: &'a Screen, hide_cursor: bool, line: ItemView) {
    let style = &screen.config.style;
    let is_line_sel = screen.cursor == line.item_index;

    let area_sel = area_selection_highlight(style, &line);
    let line_sel = line_selection_highlight(style, &line, is_line_sel);
    let bg = area_sel.patch(line_sel);

    layout.horizontal(Some(UiItem::Style(bg)), OPTS.fill_x(), |layout| {
        let gutter_char = if !hide_cursor && line.highlighted {
            gutter_char(style, is_line_sel, bg)
        } else {
            (" ".into(), Style::new())
        };

        layout_span(layout, gutter_char);

        let item = &screen.items[line.item_index];
        ui::item::layout_item(layout, item, &screen.config, bg);

        // Add ellipsis indicator for collapsed sections
        if screen.is_collapsed(item) {
            layout_span(layout, ("…".into(), bg));
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

fn line_selection_highlight(style: &StyleConfig, line: &ItemView, selected_line: bool) -> Style {
    if line.highlighted && selected_line {
        Style::from(&style.selection_line)
    } else {
        Style::new()
    }
}

fn area_selection_highlight(style: &StyleConfig, line: &ItemView) -> Style {
    if line.highlighted {
        Style::from(&style.selection_area)
    } else {
        Style::new()
    }
}
