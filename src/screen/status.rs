use super::Screen;
use crate::{
    diff, git,
    items::{self, Item},
    status::BranchStatus,
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
    let status = git::status();

    let untracked = status
        .files
        .iter()
        .filter(|file| file.is_untracked())
        .map(|file| Item {
            display: Some((
                file.path.clone(),
                Style::new().fg(theme::CURRENT_THEME.unstaged_file),
            )),
            depth: 1,
            untracked_file: Some(file.path.clone()),
            ..Default::default()
        })
        .collect::<Vec<_>>();

    let unmerged = status
        .files
        .iter()
        .filter(|file| file.is_unmerged())
        .map(|file| Item {
            display: Some((
                file.path.clone(),
                Style::new().fg(theme::CURRENT_THEME.unmerged_file),
            )),
            depth: 1,
            untracked_file: Some(file.path.clone()),
            ..Default::default()
        })
        .collect::<Vec<_>>();

    iter::once(Item {
        display: Some((format_branch_status(&status.branch_status), Style::new())),
        ..Default::default()
    })
    .chain(if untracked.is_empty() {
        None
    } else {
        Some(Item {
            display: Some((
                "\nUntracked files".to_string(),
                Style::new().fg(theme::CURRENT_THEME.section),
            )),
            section: true,
            depth: 0,
            ..Default::default()
        })
    })
    .chain(
        if unmerged.is_empty() {
            None
        } else {
            Some(Item {
                display: Some((
                    "\nUnmerged".to_string(),
                    Style::new().fg(theme::CURRENT_THEME.section),
                )),
                section: true,
                depth: 0,
                ..Default::default()
            })
        }
        .into_iter(),
    )
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

fn format_branch_status(status: &BranchStatus) -> String {
    if status.ahead_behind_count == 0 {
        format!(
            "On branch {}\nYour branch is up to date with '{}'.",
            status.local, status.remote
        )
    } else if status.ahead_behind_count > 0 {
        format!(
            "On branch {}\nYour branch is ahead of '{}' by {} commit.",
            status.local, status.remote, status.ahead_behind_count
        )
    } else {
        format!(
            "On branch {}\nYour branch is behind '{}' by {} commit.",
            status.local, status.remote, -status.ahead_behind_count
        )
    }
}

fn create_status_section_items<'a>(header: &str, diff: diff::Diff) -> impl Iterator<Item = Item> {
    if diff.deltas.is_empty() {
        None
    } else {
        Some(Item {
            display: Some((
                format!("{} ({})", header, diff.deltas.len()),
                Style::new().fg(theme::CURRENT_THEME.section),
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
