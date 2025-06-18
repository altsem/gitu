use super::Screen;
use crate::{
    config::Config,
    error::Error,
    git::{self, diff::Diff},
    git2_opts,
    items::{self, hash, Item},
    item_data::{ItemData, SectionHeader},
    Res,
};
use git2::Repository;
use ratatui::prelude::Size;
use std::{path::PathBuf, rc::Rc};

pub(crate) fn create(config: Rc<Config>, repo: Rc<Repository>, size: Size) -> Res<Screen> {
    Screen::new(
        Rc::clone(&config),
        size,
        Box::new(move || {
            let statuses = repo
                .statuses(Some(&mut git2_opts::status(&repo)?))
                .map_err(Error::GitStatus)?;

            let untracked_files = statuses
                .iter()
                .filter(|status| status.status().is_wt_new())
                .map(|status| PathBuf::from(status.path().unwrap()))
                .collect::<Vec<_>>();

            let untracked = items_list(untracked_files.clone());

            let items = if let Some(rebase) = git::rebase_status(&repo)? {
                vec![Item {
                    id: hash("rebase_status"),
                    data: Some(ItemData::Header(SectionHeader::Rebase(
                        rebase.head_name,
                        rebase.onto,
                    ))),
                    ..Default::default()
                }]
                .into_iter()
            } else if let Some(merge) = git::merge_status(&repo)? {
                vec![Item {
                    id: hash("merge_status"),
                    data: Some(ItemData::Header(SectionHeader::Merge(merge.head))),
                    ..Default::default()
                }]
                .into_iter()
            } else if let Some(revert) = git::revert_status(&repo)? {
                vec![Item {
                    id: hash("revert_status"),
                    data: Some(ItemData::Header(SectionHeader::Revert(revert.head))),
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
                        id: hash("untracked"),
                        section: true,
                        depth: 0,
                        data: Some(ItemData::AllUntracked(untracked_files)),
                        ..Default::default()
                    },
                ]
            })
            .chain(untracked)
            .chain(create_status_section_items(
                Rc::clone(&config),
                "unstaged_changes",
                Some(ItemData::AllUnstaged),
                &Rc::new(git::diff_unstaged(repo.as_ref())?),
            ))
            .chain(create_status_section_items(
                Rc::clone(&config),
                "staged_changes",
                Some(ItemData::AllStaged),
                &Rc::new(git::diff_staged(repo.as_ref())?),
            ))
            .chain(create_stash_list_section_items(
                Rc::clone(&config),
                repo.as_ref(),
                "stashes",
            ))
            .chain(create_log_section_items(
                Rc::clone(&config),
                repo.as_ref(),
                "recent_commits",
            ))
            .collect();

            Ok(items)
        }),
    )
}

fn items_list(files: Vec<PathBuf>) -> Vec<Item> {
    files
        .into_iter()
        .map(|path| Item {
            id: hash(&path),
            depth: 1,
            data: Some(ItemData::File(path)),
            ..Default::default()
        })
        .collect::<Vec<_>>()
}

fn branch_status_items(repo: &Repository) -> Res<Vec<Item>> {
    let Ok(head) = repo.head() else {
        return Ok(vec![Item {
            id: hash("branch_status"),
            section: true,
            depth: 0,
            data: Some(ItemData::Header(SectionHeader::NoBranch)),
            ..Default::default()
        }]);
    };

    let head_shorthand = head.shorthand().unwrap().to_string();

    let mut items = vec![Item {
        id: hash("branch_status"),
        section: true,
        depth: 0,
        data: Some(ItemData::Header(SectionHeader::OnBranch(head_shorthand))),
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
            id: hash("branch_status"),
            depth: 1,
            unselectable: true,
            data: Some(ItemData::Header(SectionHeader::UpstreamGone(
                upstream_shortname,
            ))),
            ..Default::default()
        });
        return Ok(items);
    };

    let (ahead, behind) = repo
        .graph_ahead_behind(head.target().unwrap(), upstream_id)
        .map_err(Error::GitStatus)?;

    items.push(Item {
        id: hash("branch_status"),
        depth: 1,
        unselectable: true,
        data: Some(ItemData::BranchStatus(upstream_shortname, ahead, behind)),
        ..Default::default()
    });

    Ok(items)
}

fn create_status_section_items<'a>(
    config: Rc<Config>,
    snake_case_header: &str,
    header_data: Option<ItemData>,
    diff: &'a Rc<Diff>,
) -> impl Iterator<Item = Item> + 'a {
    if diff.file_diffs.is_empty() {
        vec![]
    } else {
        vec![
            Item {
                unselectable: true,
                depth: 0,
                ..Default::default()
            },
            Item {
                id: hash(snake_case_header),
                section: true,
                depth: 0,
                data: header_data,
                ..Default::default()
            },
        ]
    }
    .into_iter()
    .chain(items::create_diff_items(diff, 1, true))
}

fn capitalize(str: &str) -> String {
    let first: String = str.chars().take(1).flat_map(char::to_uppercase).collect();
    let rest: String = str.chars().skip(1).collect();
    format!("{first}{rest}")
}

fn create_stash_list_section_items<'a>(
    config: Rc<Config>,
    repo: &Repository,
    snake_case_header: &str,
) -> impl Iterator<Item = Item> + 'a {
    let stashes = items::stash_list(repo, 10).unwrap();
    if stashes.is_empty() {
        vec![]
    } else {
        let style = &config.style;
        vec![
            items::blank_line(),
            Item {
                id: hash(snake_case_header),
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
    snake_case_header: &str,
) -> impl Iterator<Item = Item> + 'a {
    let style = &config.style;
    [
        Item {
            depth: 0,
            unselectable: true,
            ..Default::default()
        },
        Item {
            id: hash(snake_case_header),
            section: true,
            depth: 0,
            ..Default::default()
        },
    ]
    .into_iter()
    .chain(items::log(repo, 10, None, None).unwrap())
}
