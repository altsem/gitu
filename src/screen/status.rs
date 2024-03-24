use super::Screen;
use crate::{
    config::Config,
    git::{self, diff::Diff},
    git2_opts,
    items::{self, Item, TargetData},
    Res,
};
use git2::Repository;
use ratatui::{
    prelude::Rect,
    text::{Line, Span},
};
use std::{path::PathBuf, rc::Rc};

pub(crate) fn create(config: Rc<Config>, repo: Rc<Repository>, size: Rect) -> Res<Screen> {
    Screen::new(
        Rc::clone(&config),
        size,
        Box::new(move || {
            let style = &config.style;
            let statuses = repo.statuses(Some(&mut git2_opts::status(&repo)?))?;

            let untracked_files = statuses
                .iter()
                .filter(|status| status.status().is_wt_new())
                .map(|status| PathBuf::from(status.path().unwrap()))
                .collect::<Vec<_>>();

            let unmerged_files = statuses
                .iter()
                .filter(|status| status.status().is_conflicted())
                .map(|status| PathBuf::from(status.path().unwrap()))
                .collect::<Vec<_>>();

            let untracked = items_list(&config, untracked_files.clone());
            let unmerged = items_list(&config, unmerged_files);

            let items = if let Some(rebase) = git::rebase_status(&repo)? {
                vec![Item {
                    id: "rebase_status".into(),
                    display: Line::styled(
                        format!("Rebasing {} onto {}", rebase.head_name, &rebase.onto),
                        &style.section_header,
                    ),
                    ..Default::default()
                }]
                .into_iter()
            } else if let Some(merge) = git::merge_status(&repo)? {
                vec![Item {
                    id: "merge_status".into(),
                    display: Line::styled(
                        format!("Merging {}", &merge.head),
                        &style.section_header,
                    ),
                    ..Default::default()
                }]
                .into_iter()
            } else {
                branch_status_items(&config, &repo)?.into_iter()
            }
            .chain(if untracked.is_empty() {
                vec![]
            } else {
                vec![
                    items::blank_line(),
                    Item {
                        id: "untracked".into(),
                        display: Line::styled("Untracked files", &style.section_header),
                        section: true,
                        depth: 0,
                        target_data: Some(TargetData::AllUntracked(untracked_files)),
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
                        display: Line::styled("Unmerged", &style.section_header),
                        section: true,
                        depth: 0,
                        ..Default::default()
                    },
                ]
            })
            .chain(unmerged)
            .chain(create_status_section_items(
                Rc::clone(&config),
                "Unstaged changes",
                Some(TargetData::AllUnstaged),
                &git::diff_unstaged(&config, repo.as_ref())?,
            ))
            .chain(create_status_section_items(
                Rc::clone(&config),
                "Staged changes",
                Some(TargetData::AllStaged),
                &git::diff_staged(&config, repo.as_ref())?,
            ))
            .chain(create_stash_list_section_items(
                Rc::clone(&config),
                repo.as_ref(),
                "Stashes",
            ))
            .chain(create_log_section_items(
                Rc::clone(&config),
                repo.as_ref(),
                "Recent commits",
            ))
            .collect();

            Ok(items)
        }),
    )
}

fn items_list(config: &Config, files: Vec<PathBuf>) -> Vec<Item> {
    let style = &config.style;
    files
        .into_iter()
        .map(|path| Item {
            id: path.to_string_lossy().to_string().into(),
            display: Line::styled(path.to_string_lossy().to_string(), &style.file_header),
            depth: 1,
            target_data: Some(items::TargetData::File(path)),
            ..Default::default()
        })
        .collect::<Vec<_>>()
}

fn branch_status_items(config: &Config, repo: &Repository) -> Res<Vec<Item>> {
    let style = &config.style;
    let Ok(head) = repo.head() else {
        return Ok(vec![Item {
            id: "branch_status".into(),
            display: Line::styled("No branch", &style.section_header),
            section: true,
            depth: 0,
            ..Default::default()
        }]);
    };

    let mut items = vec![Item {
        id: "branch_status".into(),
        display: Line::styled(
            format!("On branch {}", head.shorthand().unwrap()),
            &style.section_header,
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
            Line::raw(format!("Your branch is up to date with '{}'.", upstream_shortname))
        } else if ahead > 0 && behind == 0 {
            Line::raw(format!(
                "Your branch is ahead of '{}' by {} commit.",
                upstream_shortname, ahead
            ))
        } else if ahead == 0 && behind > 0 {
            Line::raw(format!(
                "Your branch is behind '{}' by {} commit.",
                upstream_shortname, behind
            ))
        } else {
            Line::raw(format!("Your branch and '{}' have diverged,\nand have {} and {} different commits each, respectively.", upstream_shortname, ahead, behind))
        },
        depth: 1,
        unselectable: true,
        ..Default::default()
    });

    Ok(items)
}

fn create_status_section_items<'a>(
    config: Rc<Config>,
    header: &str,
    header_data: Option<TargetData>,
    diff: &'a Diff,
) -> impl Iterator<Item = Item> + 'a {
    let style = &config.style;
    if diff.deltas.is_empty() {
        vec![]
    } else {
        vec![
            Item {
                display: Line::raw(""),
                unselectable: true,
                depth: 0,
                ..Default::default()
            },
            Item {
                id: header.to_string().into(),
                display: Line::from(vec![
                    Span::styled(header.to_string(), &style.section_header),
                    format!(" ({})", diff.deltas.len()).into(),
                ]),
                section: true,
                depth: 0,
                target_data: header_data,
                ..Default::default()
            },
        ]
    }
    .into_iter()
    .chain(items::create_diff_items(config, diff, &1, true))
}

fn create_stash_list_section_items<'a>(
    config: Rc<Config>,
    repo: &Repository,
    header: &str,
) -> impl Iterator<Item = Item> + 'a {
    let stashes = items::stash_list(&config, repo, 10).unwrap();
    if stashes.is_empty() {
        vec![]
    } else {
        let style = &config.style;
        vec![
            items::blank_line(),
            Item {
                id: header.to_string().into(),
                display: Line::styled(header.to_string(), &style.section_header),
                section: true,
                depth: 0,
                ..Default::default()
            },
        ]
    }
    .into_iter()
    .chain(stashes)
}

fn create_log_section_items<'a>(
    config: Rc<Config>,
    repo: &Repository,
    header: &str,
) -> impl Iterator<Item = Item> + 'a {
    let style = &config.style;
    [
        Item {
            display: Line::raw(""),
            depth: 0,
            unselectable: true,
            ..Default::default()
        },
        Item {
            id: header.to_string().into(),
            display: Line::styled(header.to_string(), &style.section_header),
            section: true,
            depth: 0,
            ..Default::default()
        },
    ]
    .into_iter()
    .chain(items::log(&config, repo, 10, None).unwrap())
}
