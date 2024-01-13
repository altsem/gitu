use super::IssuedCommand;
use super::Item;
use std::collections::HashSet;
use std::io;
use std::process::Command;

pub(crate) struct Screen {
    pub(crate) cursor: usize,
    pub(crate) scroll: u16,
    pub(crate) refresh_items: Box<dyn Fn() -> Vec<Item>>,
    pub(crate) items: Vec<Item>,
    pub(crate) collapsed: HashSet<Item>,
    pub(crate) command: Option<IssuedCommand>,
}

impl Screen {
    pub(crate) fn issue_command(
        &mut self,
        input: &[u8],
        command: Command,
    ) -> Result<(), io::Error> {
        if !self.command.as_mut().is_some_and(|cmd| cmd.is_running()) {
            self.command = Some(IssuedCommand::spawn(input, command)?);
        }

        Ok(())
    }

    pub(crate) fn handle_command_output(&mut self) {
        if let Some(cmd) = &mut self.command {
            cmd.read_command_output_to_buffer();

            if cmd.just_finished() {
                self.items = (self.refresh_items)();
            }
        }
    }

    pub(crate) fn select_next(&mut self) {
        self.cursor = self.find_next()
    }

    pub(crate) fn find_next(&mut self) -> usize {
        self.collapsed_items_iter()
            .find(|(i, item)| i > &self.cursor && item.diff_line.is_none())
            .map(|(i, _item)| i)
            .unwrap_or(self.cursor)
    }

    pub(crate) fn select_previous(&mut self) {
        self.cursor = self
            .collapsed_items_iter()
            .filter(|(i, item)| i < &self.cursor && item.diff_line.is_none())
            .last()
            .map(|(i, _item)| i)
            .unwrap_or(self.cursor)
    }

    pub(crate) fn scroll_start(&self) -> usize {
        self.collapsed_lines_items_iter()
            .find(|(_line, i, _item)| i == &self.cursor)
            .map(|(line, _i, _item)| line)
            .unwrap()
    }

    pub(crate) fn scroll_end(&self) -> usize {
        self.collapsed_lines_items_iter()
            .filter(|(_line, i, item)| i >= &self.cursor && item.diff_line.is_none())
            .map(|(line, _i, _item)| line)
            .take(2)
            .last()
            .unwrap()
    }

    fn collapsed_lines_items_iter(&self) -> impl Iterator<Item = (usize, usize, &Item)> {
        self.collapsed_items_iter().scan(0, |lines, (i, item)| {
            let line = *lines;

            *lines += item
                .display
                .as_ref()
                .map(|item| item.0.lines().count())
                .unwrap_or(1);

            Some((line, i, item))
        })
    }

    pub(crate) fn toggle_section(&mut self) {
        let selected = &self.items[self.cursor];

        if selected.section {
            if self.collapsed.contains(selected) {
                self.collapsed.remove(selected);
            } else {
                self.collapsed.insert(selected.clone());
            }
        }
    }

    pub(crate) fn clamp_selected(&mut self) {
        self.cursor = self.cursor.clamp(0, self.items.len().saturating_sub(1))
    }

    pub(crate) fn refresh_items(&mut self) {
        self.items = (self.refresh_items)();
    }

    pub(crate) fn collapsed_items_iter<'a>(&'a self) -> impl Iterator<Item = (usize, &'a Item)> {
        self.items
            .iter()
            .enumerate()
            .scan(None, |collapse_depth, (i, next)| {
                if collapse_depth.is_some_and(|depth| depth < next.depth) {
                    return Some(None);
                }

                *collapse_depth = if next.section && self.collapsed.contains(next) {
                    Some(next.depth)
                } else {
                    None
                };

                Some(Some((i, next)))
            })
            .flatten()
    }
}
