use super::Screen;
use crate::{
    diff, git,
    items::{self, Item},
};
use ratatui::style::Style;
use std::iter;

pub(crate) fn create(size: (u16, u16), args: Vec<String>) -> Screen {
    let args_clone = args.clone();
    Screen::new(
        size,
        Box::new(move || create_show_items(args_clone.clone()).collect()),
    )
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
