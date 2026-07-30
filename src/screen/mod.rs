use crate::config::StyleConfig;
use crate::style::Style;
use crate::ui::layout::{LayoutTree, opts};
use crate::ui::{UiTree, layout_span};
use crate::{item_data::ItemData, ui};

use crate::{Res, config::Config, items::hash};

use super::Item;
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashSet;
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
    /// Memoized `item_height`, indexed like `items`. Dropped by `invalidate`.
    item_heights: RefCell<Vec<Option<u16>>>,
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
            item_heights: RefCell::new(vec![]),
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
        screen.invalidate();

        screen.cursor = screen
            .find_first_hunk()
            .or_else(|| screen.find_item(|item| !item.unselectable))
            .unwrap_or(0);

        Ok(screen)
    }

    fn find_first_hunk(&mut self) -> Option<usize> {
        self.find_item(|item| !item.unselectable && matches!(item.data, ItemData::Hunk { .. }))
    }

    pub(crate) fn select_next(&mut self, nav_mode: NavMode) {
        self.cursor = self.find_next(nav_mode);
        self.scroll_fit_end();
        self.scroll_fit_start();
    }

    fn scroll_fit_start(&mut self) {
        let Some(line_of_item) = self.line_of_item(self.cursor) else {
            return;
        };
        let top = line_of_item.saturating_sub(self.get_selected_item().depth);
        if top < self.scroll {
            self.scroll = top;
        }
    }

    fn scroll_fit_end(&mut self) {
        let Some(line_of_item) = self.line_of_item(self.cursor) else {
            return;
        };

        let depth = self.items[self.cursor].depth;

        let selection_height = self
            .visible_items()
            .skip_while(|&item_i| item_i < self.cursor)
            .take_while(|&item_i| item_i == self.cursor || depth < self.items[item_i].depth)
            .map(|item_i| self.item_height(item_i))
            .sum::<usize>();

        let Some(last_item_line) = (line_of_item + selection_height).checked_sub(1) else {
            return;
        };

        let last = BOTTOM_CONTEXT_LINES + last_item_line;

        let end_line = self.size.1.saturating_sub(1) as usize;
        if last > end_line + self.scroll {
            self.scroll = last - end_line;
        }
    }

    pub(crate) fn find_next(&mut self, nav_mode: NavMode) -> usize {
        self.visible_items()
            .skip_while(|&item_i| item_i <= self.cursor)
            .find(|&item_i| self.nav_filter(item_i, nav_mode))
            .unwrap_or(self.cursor)
    }

    /// Whether `item_i` may be selected, disregarding whether it is hidden by
    /// a collapsed ancestor.
    fn nav_filter(&self, item_i: usize, nav_mode: NavMode) -> bool {
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

    fn is_selectable(&self, item_i: usize, nav_mode: NavMode) -> bool {
        self.is_visible(item_i) && self.nav_filter(item_i, nav_mode)
    }

    pub(crate) fn select_previous(&mut self, nav_mode: NavMode) {
        self.cursor = self.find_previous(nav_mode);
        self.scroll_fit_start();
    }

    fn find_previous(&mut self, nav_mode: NavMode) -> usize {
        self.visible_items()
            .take_while(|&item_i| item_i < self.cursor)
            .filter(|&item_i| self.nav_filter(item_i, nav_mode))
            .last()
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

            // Only the toggled section's own height changes, by gaining or
            // losing its `…`. Everything else lays out from its own content at
            // an unchanged width, so those memoized heights still hold.
            self.item_heights.get_mut()[self.cursor] = None;
        }

        self.clamp_scroll();
        Ok(())
    }

    pub(crate) fn refresh(&mut self) -> Res<()> {
        self.items = (self.refresh_items)()?;
        self.invalidate();
        self.update_cursor();
        Ok(())
    }

    pub(crate) fn resize(&mut self, w: u16, h: u16) -> Res<()> {
        self.size = (w, h);
        self.invalidate();
        self.update_cursor();
        Ok(())
    }

    fn update_cursor(&mut self) {
        // Nothing is selectable (e.g. the log of a branch with no commits).
        // Reset the cursor to a valid sentinel rather than positioning it,
        // which would index into a screen with no lines and panic (#262).
        if self.is_empty() {
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

    /// Drops everything derived from `items`, `collapsed` and `size`.
    /// The accessors below recompute what they need, when they need it.
    fn invalidate(&mut self) {
        let item_heights = self.item_heights.get_mut();
        item_heights.clear();
        item_heights.resize(self.items.len(), None);

        self.clamp_scroll();
    }

    // TODO This scan is not optimal, should save an index of it somehow
    /// Items in order, skipping those hidden by a collapsed ancestor.
    fn visible_items(&self) -> impl Iterator<Item = usize> {
        self.items
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

                Some(Some(i))
            })
            .flatten()
    }

    fn is_visible(&self, item_i: usize) -> bool {
        self.visible_items()
            .take_while(|&i| i <= item_i)
            .last()
            .is_some_and(|i| i == item_i)
    }

    /// How many lines `item_i` occupies once laid out at the current width.
    fn item_height(&self, item_i: usize) -> usize {
        if let Some(height) = self.item_heights.borrow()[item_i] {
            return height as usize;
        }

        // TODO A new allocation per measured item seems wasteful
        let mut layout = LayoutTree::new();
        let view = ItemView {
            item_index: item_i,
            highlighted: false,
        };
        layout_item(&mut layout, self, false, view);

        let height = layout
            .compute([self.size.0, self.size.1])
            .iter()
            .map(|item| item.pos[1] + item.size[1])
            .max()
            .unwrap_or(0);

        self.item_heights.borrow_mut()[item_i] = Some(height);
        height as usize
    }

    /// The item rendered at `line`, counting from the top of the content
    /// (not the top of the screen). `None` once past the last line.
    fn item_at_line(&self, line: usize) -> Option<usize> {
        let mut remaining = line;

        self.visible_items().find(|&item_i| {
            let height = self.item_height(item_i);
            if remaining < height {
                return true;
            }

            remaining -= height;
            false
        })
    }

    /// The first line of `item_i`, or `None` if it isn't rendered at all.
    fn line_of_item(&self, item_i: usize) -> Option<usize> {
        if item_i >= self.items.len() {
            return None;
        }

        let mut line = 0;

        self.visible_items()
            .find(|&i| {
                if i == item_i {
                    return true;
                }

                line += self.item_height(i);
                false
            })
            .map(|_| line)
    }

    /// Total number of lines, but gives up counting at `limit`. Callers only
    /// ever need to compare the total against something near the bottom of the
    /// viewport, and this way the items below it are never laid out.
    ///
    /// The result is exact when below `limit`, and at least `limit` otherwise.
    fn lines_up_to(&self, limit: usize) -> usize {
        let mut lines = 0;

        for item_i in self.visible_items() {
            if lines >= limit {
                break;
            }

            lines += self.item_height(item_i);
        }

        lines
    }

    /// Whether the screen renders no lines at all, e.g. the log of a branch
    /// with no commits.
    fn is_empty(&self) -> bool {
        !self
            .visible_items()
            .any(|item_i| self.item_height(item_i) > 0)
    }

    fn is_cursor_off_screen(&self) -> bool {
        !self.item_views(self.size).any(|item| item.highlighted)
    }

    fn move_cursor_to_screen_center(&mut self) {
        let half_screen = self.size.1 as usize / 2;
        let center = self.scroll + half_screen;
        let center = center.min(self.lines_up_to(center + 1).saturating_sub(1));

        if let Some(item_i) = self.item_at_line(center) {
            self.cursor = item_i;
        }
    }

    fn clamp_cursor(&mut self) {
        self.cursor = self.cursor.clamp(0, self.items.len().saturating_sub(1));
    }

    fn clamp_scroll(&mut self) {
        // The line count is only needed to tell whether `scroll` is too far
        // down, so count no further than the point that settles it.
        let enough = (self.scroll + self.size.1 as usize)
            .saturating_sub(BOTTOM_CONTEXT_LINES)
            .max(self.scroll + 1);

        let len = self.lines_up_to(enough);
        self.scroll = self.scroll.min(self.max_scroll_with_context(len));
    }

    /// Given `len` lines of content, the furthest down the screen may scroll.
    fn max_scroll_with_context(&self, len: usize) -> usize {
        if len == 0 {
            return 0;
        }

        let max_scroll = len.saturating_sub(self.size.1 as usize);
        let max_scroll = max_scroll.saturating_add(BOTTOM_CONTEXT_LINES);
        max_scroll.min(len.saturating_sub(1))
    }

    fn move_from_unselectable(&mut self, nav_mode: NavMode) {
        if !self.is_selectable(self.cursor, nav_mode) {
            self.select_previous(nav_mode);
        }
        if !self.is_selectable(self.cursor, nav_mode) {
            self.select_next(nav_mode);
        }
    }

    pub(crate) fn move_cursor_to_screen_line(&mut self, screen_line: usize) {
        let Some(new_cursor) = self.item_at_line(screen_line + self.scroll) else {
            return;
        };
        if self.cursor == new_cursor {
            return;
        }

        let old_cursor = self.cursor;
        self.cursor = new_cursor;

        let nav_mode = self.selected_item_nav_mode();
        self.move_from_unselectable(nav_mode);

        if !self.is_selectable(self.cursor, nav_mode) {
            // There was no selectable item, put the cursor back.
            self.cursor = old_cursor;
        } else {
            // Use minimal scrolling to keep the cursor visible.
            self.scroll_fit_start();
        }
    }

    pub(crate) fn move_cursor_to_top(&mut self) {
        if let Some(first) = self.find_item(|item| !item.unselectable) {
            self.cursor = first;
            self.scroll = 0;
        }
    }

    pub(crate) fn move_cursor_to_bottom(&mut self) {
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
        self.visible_items()
            .find(|&item_i| predicate(&self.items[item_i]))
    }

    fn rfind_item<P: Fn(&Item) -> bool>(&self, predicate: P) -> Option<usize> {
        self.visible_items()
            .filter(|&item_i| predicate(&self.items[item_i]))
            .last()
    }

    pub(crate) fn is_valid_screen_line(&self, screen_line: usize) -> bool {
        let Some(target_item_i) = self.item_at_line(screen_line + self.scroll) else {
            return false;
        };
        self.nav_filter(target_item_i, NavMode::IncludeSubLines)
    }

    fn item_views(&self, area: (u16, u16)) -> impl Iterator<Item = ItemView> {
        let first_visible_item = self
            .item_at_line(self.scroll)
            .unwrap_or(self.items.len().saturating_sub(1));

        // Scanning starts at the cursor when it is above the viewport, so that
        // the highlight of an item whose children reach into view is known.
        // Those extra items are dropped again by `context_offset`.
        let scan_start_item = first_visible_item.min(self.cursor);
        let scan_end_item = self
            .item_at_line(self.scroll + area.1 as usize)
            .unwrap_or(self.items.len());

        let context_offset = self
            .visible_items()
            .take_while(|&item_i| item_i < first_visible_item)
            .filter(|&item_i| item_i >= scan_start_item)
            .count();

        self.visible_items()
            .skip_while(move |&item_i| item_i < scan_start_item)
            .take_while(move |&item_i| item_i < scan_end_item)
            .scan(None, move |highlight_depth, item_index| {
                let item = &self.items[item_index];
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
    layout.col(opts().fill_x(), |layout| {
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

    layout.row_with(bg, opts().fill_x(), |layout| {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::init_test_config;

    fn screen_of(item_count: usize, size: (u16, u16)) -> Screen {
        let config = Arc::new(init_test_config().unwrap());

        Screen::new(
            config,
            size,
            Box::new(move || {
                Ok((0..item_count)
                    .map(|i| Item {
                        id: i as u64,
                        data: ItemData::Raw(format!("item {i}")),
                        ..Default::default()
                    })
                    .collect())
            }),
        )
        .unwrap()
    }

    fn laid_out_count(screen: &Screen) -> usize {
        screen
            .item_heights
            .borrow()
            .iter()
            .filter(|height| height.is_some())
            .count()
    }

    /// Items are laid out on demand, so a long list costs no more than a short
    /// one until something actually scrolls down to it.
    #[test]
    fn only_lays_out_what_it_reaches() {
        let mut screen = screen_of(10_000, (80, 20));
        assert!(laid_out_count(&screen) < 50, "{}", laid_out_count(&screen));

        screen.scroll_view_down(100);
        assert!(laid_out_count(&screen) < 150, "{}", laid_out_count(&screen));
    }

    /// Scrolling is allowed a couple of lines past the content, so on a screen
    /// the content doesn't fill, its center lands past the last line.
    #[test]
    fn recenter_cursor_on_a_screen_the_content_doesnt_fill() {
        let mut screen = screen_of(3, (80, 20));

        screen.scroll_view_down(2);
        screen.refresh().unwrap();

        assert_eq!(2, screen.cursor);
    }
}
