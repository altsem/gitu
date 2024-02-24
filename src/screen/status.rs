use std::{iter, rc::Rc};

use super::Screen;
use crate::{
    git::{self, diff::Diff, status::BranchStatus},
    git2_opts,
    items::{self, Item},
    theme::CURRENT_THEME,
    Config, Res,
};
use git2::Repository;
use ratatui::{prelude::Rect, style::Stylize, text::Text};

pub(crate) fn create(repo: Rc<Repository>, config: &Config, size: Rect) -> Res<Screen> {
    let config = config.clone();

    Screen::new(
        size,
        Box::new(move || {
            let statuses = repo.statuses(Some(&mut git2_opts::status(&repo)?))?;
            // TODO Replace with libgit2
            let status = git::status(&config.dir)?;

            let untracked = untracked(&statuses);
            let unmerged = unmerged(&statuses);

            let items = if let Some(rebase) = git::rebase_status(&config.dir)? {
                vec![Item {
                    id: "rebase_status".into(),
                    display: Text::from(
                        format!("Rebasing {} onto {}", rebase.head_name, &rebase.onto)
                            .fg(CURRENT_THEME.section)
                            .bold(),
                    ),
                    ..Default::default()
                }]
                .into_iter()
            } else if let Some(merge) = git::merge_status(&config.dir)? {
                vec![Item {
                    id: "merge_status".into(),
                    display: Text::from(
                        format!("Merging {}", &merge.head)
                            .fg(CURRENT_THEME.section)
                            .bold(),
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
                    items::blank_line(),
                    Item {
                        id: "untracked".into(),
                        display: Text::from(
                            "Untracked files"
                                .to_string()
                                .fg(CURRENT_THEME.section)
                                .bold(),
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
                    items::blank_line(),
                    Item {
                        id: "unmerged".into(),
                        display: Text::from(
                            "Unmerged".to_string().fg(CURRENT_THEME.section).bold(),
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
                &git::diff_unstaged(repo.as_ref())?,
            ))
            .chain(create_status_section_items(
                "Staged changes",
                &git::diff_staged(repo.as_ref())?,
            ))
            .chain(create_log_section_items(repo.as_ref(), "Recent commits"))
            .collect();

            Ok(items)
        }),
    )
}

fn untracked(statuses: &git2::Statuses<'_>) -> Vec<Item> {
    statuses
        .iter()
        .filter_map(|status| {
            if !status.status().is_wt_new() {
                return None;
            }

            let Some(path) = status.path() else {
                return None;
            };

            Some(Item {
                id: path.to_string().into(),
                display: Text::from(path.to_string().fg(CURRENT_THEME.unstaged_file).bold()),
                depth: 1,
                target_data: Some(items::TargetData::File(path.to_string())),
                ..Default::default()
            })
        })
        .collect::<Vec<_>>()
}

fn unmerged(statuses: &git2::Statuses<'_>) -> Vec<Item> {
    statuses
        .iter()
        .filter_map(|status| {
            if !status.status().is_conflicted() {
                return None;
            }

            let Some(path) = status.path() else {
                return None;
            };

            Some(Item {
                id: path.to_string().into(),
                display: Text::from(path.to_string().fg(CURRENT_THEME.unstaged_file).bold()),
                depth: 1,
                target_data: Some(items::TargetData::File(path.to_string())),
                ..Default::default()
            })
        })
        .collect::<Vec<_>>()
}

fn branch_status_items(status: &BranchStatus) -> Vec<Item> {
    match (&status.local, &status.remote) {
        (None, None) => vec![Item {
            id: "branch_status".into(),
            display: Text::from("No branch".fg(CURRENT_THEME.section).bold()),
            section: true,
            depth: 0,
            ..Default::default()
        }],
        (Some(local), maybe_remote) => Vec::from_iter(
            iter::once(Item {
                id: "branch_status".into(),
                display: Text::from(
                    format!("On branch {}", local)
                        .fg(CURRENT_THEME.section)
                        .bold(),
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
                display: Text::from(
                    format!("{} ({})", header, diff.deltas.len())
                        .fg(CURRENT_THEME.section)
                        .bold(),
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

fn create_log_section_items<'a>(
    repo: &Repository,
    header: &str,
) -> impl Iterator<Item = Item> + 'a {
    [
        Item {
            display: Text::raw(""),
            depth: 0,
            unselectable: true,
            ..Default::default()
        },
        Item {
            id: header.to_string().into(),
            display: Text::from(header.to_string().fg(CURRENT_THEME.section).bold()),
            section: true,
            depth: 0,
            ..Default::default()
        },
    ]
    .into_iter()
    .chain(items::log(repo, 5).unwrap())
}
