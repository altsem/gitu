use super::Screen;
use crate::{
    diff, git,
    items::{self, Item},
};
use ratatui::style::Style;
use std::{collections::HashSet, iter};

pub(crate) fn create(size: (u16, u16), reference: String) -> Screen {
    let r = reference.clone();
    Screen {
        cursor: 0,
        scroll: 0,
        size,
        refresh_items: Box::new(move || create_show_items(&r).collect()),
        items: create_show_items(&reference).collect(),
        collapsed: HashSet::new(),
        command: None,
    }
}

fn create_show_items(reference: &str) -> impl Iterator<Item = Item> {
    iter::once(Item {
        display: Some((git::show_summary(reference), Style::new())),
        ..Default::default()
    })
    .chain(items::create_diff_items(
        diff::Diff::parse(&git::show(reference)),
        0,
    ))
}
