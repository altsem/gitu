use ratatui::{layout::Size, text::Line};
use termwiz::{
    color::ColorAttribute,
    surface::Change,
    widgets::{RenderArgs, Widget},
};

use crate::{
    config::Config,
    items::{hash, TargetData},
    Res,
};

use super::Item;
use std::{collections::HashSet, rc::Rc};

pub(crate) mod log;
pub(crate) mod show;
pub(crate) mod show_refs;
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
                && matches!(
                    self.at_line(line_i).target_data,
                    Some(TargetData::Hunk { .. })
                )
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
                let target_data = item.target_data.as_ref();
                let is_hunk_line =
                    target_data.is_some_and(|d| matches!(d, TargetData::HunkLine { .. }));

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

        match self.get_selected_item().target_data {
            Some(TargetData::HunkLine { .. }) => NavMode::IncludeHunkLines,
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
        !self
            .line_views((self.size.width as usize, self.size.height as usize))
            .any(|line| line.highlighted)
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

    fn is_collapsed(&self, item: &Item) -> bool {
        self.collapsed.contains(&item.id)
    }

    pub(crate) fn get_selected_item(&self) -> &Item {
        &self.items[self.line_index[self.cursor]]
    }

    fn line_views(&self, (_, height): (usize, usize)) -> impl Iterator<Item = LineView> {
        let scan_start = self.scroll.min(self.cursor);
        let scan_end = (self.scroll + height).min(self.line_index.len());
        let scan_highlight_range = scan_start..(scan_end);
        let context_lines = self.scroll - scan_start;

        self.line_index[scan_highlight_range]
            .iter()
            .scan(None, |highlight_depth, item_i| {
                let item = &self.items[*item_i];
                if self.line_index[self.cursor] == *item_i {
                    *highlight_depth = Some(item.depth);
                } else if highlight_depth.is_some_and(|s| s >= item.depth) {
                    *highlight_depth = None;
                };

                Some(LineView {
                    item_index: *item_i,
                    item,
                    display: &item.display,
                    highlighted: highlight_depth.is_some(),
                })
            })
            .skip(context_lines)
    }
}

struct LineView<'a> {
    item_index: usize,
    item: &'a Item,
    display: &'a Line<'a>,
    highlighted: bool,
}

impl Widget for Screen {
    fn render(&mut self, args: &mut RenderArgs) {
        args.surface
            .add_change(Change::ClearScreen(ColorAttribute::Default));

        for (line_index, line) in self.line_views(args.surface.dimensions()).enumerate() {
            // TODO Render the screen(buffer) like before.
            // - apply colors from configuration
            // - lines will now naturally wrap (unlike with Ratatui's `Line`)
            // - Hopefully styles could be applied here from the output of highlight.rs: `(Range<usize>, Style)`
            //   That might simplify things from working with Text/Line/Span of Ratatui
            args.surface.add_change(format!("{}\r\n", line.display));
        }
    }
}
