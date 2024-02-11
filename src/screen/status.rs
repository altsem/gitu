use std::iter;

use super::Screen;
use crate::{
    git::{self, diff::Diff, status::BranchStatus},
    items::{self, Item},
    theme::CURRENT_THEME,
    Config, Res,
};
use ratatui::{
    prelude::Rect,
    style::{Style, Stylize},
    text::Text,
};

pub(crate) fn create(config: &Config, size: Rect) -> Res<Screen> {
    let config = config.clone();

    Screen::new(
        size,
        Box::new(move || {
            let status = git::status(&config.dir)?;
            let untracked = untracked(&status);
            let unmerged = unmerged(&status);

            let items = if let Some(rebase) = git::rebase_status(&config.dir)? {
                vec![Item {
                    id: "rebase_status".into(),
                    display: Text::styled(
                        format!("Rebasing {} onto {}", rebase.head_name, &rebase.onto),
                        Style::new().fg(CURRENT_THEME.section).bold(),
                    ),
                    ..Default::default()
                }]
                .into_iter()
            } else if let Some(merge) = git::merge_status(&config.dir)? {
                vec![Item {
                    id: "merge_status".into(),
                    display: Text::styled(
                        format!("Merging {}", &merge.head),
                        Style::new().fg(CURRENT_THEME.section).bold(),
                    ),
                    ..Default::default()
                }]
                .into_iter()
            } else {
                branch_status_items(&status.branch_status).into_iter()
            }
            .chain(if untracked.is_empty() {
                vec![]
            } else {
                vec![
                    blank_line(),
                    Item {
                        id: "untracked".into(),
                        display: Text::styled(
                            "Untracked files".to_string(),
                            Style::new().fg(CURRENT_THEME.section).bold(),
                        ),
                        section: true,
                        depth: 0,
                        ..Default::default()
                    },
                ]
            })
            .chain(untracked)
            .chain(if unmerged.is_empty() {
                vec![]
            } else {
                vec![
                    blank_line(),
                    Item {
                        id: "unmerged".into(),
                        display: Text::styled(
                            "Unmerged".to_string(),
                            Style::new().fg(CURRENT_THEME.section).bold(),
                        ),
                        section: true,
                        depth: 0,
                        ..Default::default()
                    },
                ]
            })
            .chain(unmerged)
            .chain(create_status_section_items(
                "Unstaged changes",
                &git::diff_unstaged(&config.dir)?,
            ))
            .chain(create_status_section_items(
                "Staged changes",
                &git::diff_staged(&config.dir)?,
            ))
            .chain(create_log_section_items(
                "Recent commits",
                &git::log_recent(&config.dir)?,
            ))
            .collect();

            Ok(items)
        }),
    )
}

fn blank_line() -> Item {
    Item {
        display: Text::raw(""),
        depth: 0,
        unselectable: true,
        ..Default::default()
    }
}
fn untracked(status: &git::status::Status) -> Vec<Item> {
    status
        .files
        .iter()
        .filter(|file| file.is_untracked())
        .map(|file| Item {
            id: file.path.clone().into(),
            display: Text::styled(
                file.path.clone(),
                Style::new().fg(CURRENT_THEME.unstaged_file).bold(),
            ),
            depth: 1,
            target_data: Some(items::TargetData::File(file.path.clone())),
            ..Default::default()
        })
        .collect::<Vec<_>>()
}

fn unmerged(status: &git::status::Status) -> Vec<Item> {
    status
        .files
        .iter()
        .filter(|file| file.is_unmerged())
        .map(|file| Item {
            id: file.path.clone().into(),
            display: Text::styled(
                file.path.clone(),
                Style::new().fg(CURRENT_THEME.unmerged_file).bold(),
            ),
            depth: 1,
            target_data: Some(items::TargetData::File(file.path.clone())),
            ..Default::default()
        })
        .collect::<Vec<_>>()
}

fn branch_status_items(status: &BranchStatus) -> Vec<Item> {
    match (&status.local, &status.remote) {
        (None, None) => vec![Item {
            id: "branch_status".into(),
            display: Text::styled("No branch", Style::new().fg(CURRENT_THEME.section).bold()),
            section: true,
            depth: 0,
            ..Default::default()
        }],
        (Some(local), maybe_remote) => Vec::from_iter(
            iter::once(Item {
                id: "branch_status".into(),
                display: Text::styled(
                    format!("On branch {}", local),
                    Style::new().fg(CURRENT_THEME.section).bold(),
                ),
                section: true,
                depth: 0,
                ..Default::default()
            })
            .chain(
                maybe_remote
                    .as_ref()
                    .map(|remote| branch_status_remote_description(status, remote)),
            ),
        ),
        (None, Some(_)) => unreachable!(),
    }
}

fn branch_status_remote_description(status: &BranchStatus, remote: &str) -> Item {
    Item {
        id: "branch_status".into(),
        display: if status.ahead == 0 && status.behind == 0 {
            Text::raw(format!("Your branch is up to date with '{}'.", remote))
        } else if status.ahead > 0 && status.behind == 0 {
            Text::raw(format!(
                "Your branch is ahead of '{}' by {} commit.",
                remote, status.ahead
            ))
        } else if status.ahead == 0 && status.behind > 0 {
            Text::raw(format!(
                "Your branch is behind '{}' by {} commit.",
                remote, status.behind
            ))
        } else {
            Text::raw(format!("Your branch and '{}' have diverged,\nand have {} and {} different commits each, respectively.", remote, status.ahead, status.behind))
        },
        depth: 1,
        unselectable: true,
        ..Default::default()
    }
}

fn create_status_section_items<'a>(
    header: &str,
    diff: &'a Diff,
) -> impl Iterator<Item = Item> + 'a {
    if diff.deltas.is_empty() {
        vec![]
    } else {
        vec![
            Item {
                display: Text::raw(""),
                unselectable: true,
                depth: 0,
                ..Default::default()
            },
            Item {
                id: header.to_string().into(),
                display: Text::styled(
                    format!("{} ({})", header, diff.deltas.len()),
                    Style::new().fg(CURRENT_THEME.section).bold(),
                ),
                section: true,
                depth: 0,
                ..Default::default()
            },
        ]
    }
    .into_iter()
    .chain(items::create_diff_items(diff, &1))
}

fn create_log_section_items<'a>(header: &str, log: &'a str) -> impl Iterator<Item = Item> + 'a {
    [
        Item {
            display: Text::raw(""),
            depth: 0,
            unselectable: true,
            ..Default::default()
        },
        Item {
            id: header.to_string().into(),
            display: Text::styled(
                header.to_string(),
                Style::new().fg(CURRENT_THEME.section).bold(),
            ),
            section: true,
            depth: 0,
            ..Default::default()
        },
    ]
    .into_iter()
    .chain(items::create_log_items(log))
}
