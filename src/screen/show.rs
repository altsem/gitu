use super::ScreenData;
use crate::{
    diff::Diff,
    git,
    items::{self, Item},
};
use ratatui::style::Style;
use std::iter;

pub(crate) struct ShowData {
    summary: String,
    show: Diff,
}

impl ShowData {
    pub(crate) fn capture(args: &[&str]) -> Self {
        Self {
            summary: git::show_summary(args),
            show: git::show(args),
        }
    }
}

impl ScreenData for ShowData {
    fn items<'a>(&'a self) -> Vec<Item> {
        iter::once(Item {
            display: (self.summary.clone(), Style::new()),
            ..Default::default()
        })
        .chain(items::create_diff_items(&self.show, &0))
        .collect()
    }
}
