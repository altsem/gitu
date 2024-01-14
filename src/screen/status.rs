use super::Screen;
use crate::{
    diff, git,
    items::{self, Item},
    theme,
};
use ratatui::style::{Color, Style};
use std::{collections::HashSet, iter};

pub(crate) fn create(size: (u16, u16)) -> Screen {
    Screen {
        cursor: 0,
        scroll: 0,
        size,
        refresh_items: Box::new(|| create_status_items().collect()),
        items: create_status_items().collect(),
        collapsed: HashSet::new(),
        command: None,
    }
}

pub(crate) fn create_status_items() -> impl Iterator<Item = Item> {
    // TODO items.extend(create_status_section(&repo, None, "Untracked files"));
    let untracked = git::list_untracked()
        .lines()
        .map(|untracked| Item {
            display: Some((untracked.to_string(), Style::new().fg(theme::UNSTAGED_FILE))),
            depth: 1,
            untracked_file: Some(untracked.to_string()),
            ..Default::default()
        })
        .collect::<Vec<_>>();

    if untracked.is_empty() {
        None
    } else {
        Some(Item {
            display: Some((
                "\nUntracked files".to_string(),
                Style::new().fg(theme::SECTION),
            )),
            section: true,
            depth: 0,
            ..Default::default()
        })
    }
    .into_iter()
    .chain(untracked)
    .chain(create_status_section_items(
        "\nUnstaged changes",
        diff::Diff::parse(&git::diff_unstaged()),
    ))
    .chain(create_status_section_items(
        "\nStaged changes",
        diff::Diff::parse(&git::diff_staged()),
    ))
    .chain(create_log_section_items(
        "\nRecent commits",
        git::log_recent(),
    ))
}

fn create_status_section_items<'a>(header: &str, diff: diff::Diff) -> impl Iterator<Item = Item> {
    if diff.deltas.is_empty() {
        None
    } else {
        Some(Item {
            display: Some((
                format!("{} ({})", header, diff.deltas.len()),
                Style::new().fg(theme::SECTION),
            )),
            section: true,
            depth: 0,
            ..Default::default()
        })
    }
    .into_iter()
    .chain(items::create_diff_items(diff, 1))
}

fn create_log_section_items(header: &str, log: String) -> impl Iterator<Item = Item> {
    iter::once(Item {
        display: Some((header.to_string(), Style::new().fg(Color::Yellow))),
        section: true,
        depth: 0,
        ..Default::default()
    })
    .chain(items::create_log_items(log))
}
