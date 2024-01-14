use super::Screen;
use crate::{
    diff, git,
    items::{self, Item},
};
use ratatui::style::Style;
use std::{collections::HashSet, iter};

pub(crate) fn create(size: (u16, u16), args: Vec<String>) -> Screen {
    let args_clone = args.clone();
    Screen {
        cursor: 0,
        scroll: 0,
        size,
        refresh_items: Box::new(move || create_show_items(args_clone.clone()).collect()),
        items: create_show_items(args).collect(),
        collapsed: HashSet::new(),
        command: None,
    }
}

fn create_show_items(args: Vec<String>) -> impl Iterator<Item = Item> {
    let args = args.iter().map(String::as_str).collect::<Vec<_>>();

    iter::once(Item {
        display: Some((git::show_summary(&args), Style::new())),
        ..Default::default()
    })
    .chain(items::create_diff_items(
        diff::Diff::parse(&git::show(&args)),
        0,
    ))
}
