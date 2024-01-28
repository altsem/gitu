use crate::{
    git,
    items::{self, Item},
};
use ansi_to_tui::IntoText;
use std::iter;

pub(crate) fn create(args: &[&str]) -> Vec<Item> {
    let summary = git::show_summary(args);
    let show = git::show(args);

    iter::once(Item {
        display: summary.into_text().expect("Couldn't read ansi codes"),
        ..Default::default()
    })
    .chain(items::create_diff_items(&show, &0))
    .collect()
}
