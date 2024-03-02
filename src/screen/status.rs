use std::rc::Rc;

use super::Screen;
use crate::{
    git::{self, diff::Diff},
    git2_opts,
    items::{self, Item},
    theme::CURRENT_THEME,
    Config, Res,
};
use git2::Repository;
use ratatui::{
    prelude::Rect,
    style::Stylize,
    text::{Line, Text},
};

pub(crate) fn create(repo: Rc<Repository>, config: &Config, size: Rect) -> Res<Screen> {
    let config = config.clone();

    Screen::new(
        size,
        Box::new(move || {
            let statuses = repo.statuses(Some(&mut git2_opts::status(&repo)?))?;
            let untracked = untracked(&statuses);
            let unmerged = unmerged(&statuses);

            let items = if let Some(rebase) = git::rebase_status(&config.dir)? {
                vec![Item {
                    id: "rebase_status".into(),
                    display: Text::from(
                        format!("Rebasing {} onto {}", rebase.head_name, &rebase.onto)
                            .fg(CURRENT_THEME.section),
                    ),
                    ..Default::default()
                }]
                .into_iter()
            } else if let Some(merge) = git::merge_status(&config.dir)? {
                vec![Item {
                    id: "merge_status".into(),
                    display: Text::from(
                        format!("Merging {}", &merge.head).fg(CURRENT_THEME.section),
                    ),
                    ..Default::default()
                }]
                .into_iter()
            } else {
                branch_status_items(&repo)?.into_iter()
            }
            .chain(if untracked.is_empty() {
                vec![]
            } else {
                vec![
                    items::blank_line(),
                    Item {
                        id: "untracked".into(),
                        display: Text::from(
                            "Untracked files".to_string().fg(CURRENT_THEME.section),
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
                        display: Text::from("Unmerged".to_string().fg(CURRENT_THEME.section)),
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
                display: Text::from(path.to_string().fg(CURRENT_THEME.unstaged_file)),
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
                display: Text::from(path.to_string().fg(CURRENT_THEME.unstaged_file)),
                depth: 1,
                target_data: Some(items::TargetData::File(path.to_string())),
                ..Default::default()
            })
        })
        .collect::<Vec<_>>()
}

fn branch_status_items(repo: &Repository) -> Res<Vec<Item>> {
    let Ok(head) = repo.head() else {
        return Ok(vec![Item {
            id: "branch_status".into(),
            display: Text::from("No branch".fg(CURRENT_THEME.section)),
            section: true,
            depth: 0,
            ..Default::default()
        }]);
    };

    let mut items = vec![Item {
        id: "branch_status".into(),
        display: Text::from(
            format!("On branch {}", head.shorthand().unwrap()).fg(CURRENT_THEME.section),
        ),
        section: true,
        depth: 0,
        ..Default::default()
    }];

    let Ok(upstream) = repo.branch_upstream_name(head.name().unwrap()) else {
        return Ok(items);
    };
    let upstream_name = upstream.as_str().unwrap().to_string();
    let upstream_shortname = upstream_name
        .strip_prefix("refs/remotes/")
        .unwrap_or(&upstream_name)
        .to_string();

    let Ok(upstream_id) = repo.refname_to_id(&upstream_name) else {
        items.push(Item {
            id: "branch_status".into(),
            display: format!(
                "Your branch is based on '{}', but the upstream is gone.",
                upstream_shortname
            )
            .into(),
            depth: 1,
            unselectable: true,
            ..Default::default()
        });
        return Ok(items);
    };

    let (ahead, behind) = repo.graph_ahead_behind(head.target().unwrap(), upstream_id)?;

    items.push(Item {
        id: "branch_status".into(),
        display: if ahead == 0 && behind == 0 {
            Text::raw(format!("Your branch is up to date with '{}'.", upstream_shortname))
        } else if ahead > 0 && behind == 0 {
            Text::raw(format!(
                "Your branch is ahead of '{}' by {} commit.",
                upstream_shortname, ahead
            ))
        } else if ahead == 0 && behind > 0 {
            Text::raw(format!(
                "Your branch is behind '{}' by {} commit.",
                upstream_shortname, behind
            ))
        } else {
            Text::raw(format!("Your branch and '{}' have diverged,\nand have {} and {} different commits each, respectively.", upstream_shortname, ahead, behind))
        },
        depth: 1,
        unselectable: true,
        ..Default::default()
    });

    Ok(items)
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
                display: Text::from(Line::from(vec![
                    header.to_string().fg(CURRENT_THEME.section),
                    format!(" ({})", diff.deltas.len()).into(),
                ])),
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
            display: Text::from(header.to_string().fg(CURRENT_THEME.section)),
            section: true,
            depth: 0,
            ..Default::default()
        },
    ]
    .into_iter()
    .chain(items::log(repo, 10, None).unwrap())
}
