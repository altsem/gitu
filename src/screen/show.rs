use crate::{
    git,
    items::{self, Item},
};
use ratatui::style::Style;
use std::iter;

pub(crate) fn create(args: &[&str]) -> Vec<Item> {
    let summary = git::show_summary(args);
    let show = git::show(args);

    iter::once(Item {
        display: (summary.clone(), Style::new()),
        ..Default::default()
    })
    .chain(items::create_diff_items(&show, &0))
    .collect()
}
