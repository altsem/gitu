use super::Screen;
use crate::{
    Res,
    config::Config,
    error::Error,
    git::{self, diff::Diff, status::BranchStatus},
    item_data::{ItemData, SectionHeader},
    items::{self, Item, hash},
};
use git2::Repository;
use ratatui::prelude::Size;
use std::{hash::Hash, path::PathBuf, rc::Rc};

enum SectionID {
    RebaseStatus,
    MergeStatus,
    RevertStatus,
    Untracked,
    Stashes,
    RecentCommits,
    BranchStatus,
    UnstagedChanges,
    StagedChanges,
}

impl Hash for SectionID {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let id = match self {
            SectionID::RebaseStatus => "rebase_status",
            SectionID::MergeStatus => "merge_status",
            SectionID::RevertStatus => "revert_status",
            SectionID::Untracked => "untracked",
            SectionID::Stashes => "stashes",
            SectionID::RecentCommits => "recent_commits",
            SectionID::BranchStatus => "branch_status",
            SectionID::UnstagedChanges => "unstaged_changes",
            SectionID::StagedChanges => "staged_changes",
        };

        id.hash(state)
    }
}

pub(crate) fn create(config: Rc<Config>, repo: Rc<Repository>, size: Size) -> Res<Screen> {
    Screen::new(
        Rc::clone(&config),
        size,
        Box::new(move || {
            let status = git::status(repo.workdir().ok_or(Error::NoRepoWorkdir)?)?;
            let untracked_files = status
                .files
                .iter()
                .filter(|status| status.is_untracked())
                .map(|status| &status.path)
                .collect::<Vec<_>>();

            let untracked = items_list(&untracked_files);

            let items = if let Some(rebase) = git::rebase_status(&repo)? {
                vec![Item {
                    id: hash(SectionID::RebaseStatus),
                    data: ItemData::Header(SectionHeader::Rebase(rebase.head_name, rebase.onto)),
                    ..Default::default()
                }]
                .into_iter()
            } else if let Some(merge) = git::merge_status(&repo)? {
                vec![Item {
                    id: hash(SectionID::MergeStatus),
                    data: ItemData::Header(SectionHeader::Merge(merge.head)),
                    ..Default::default()
                }]
                .into_iter()
            } else if let Some(revert) = git::revert_status(&repo)? {
                vec![Item {
                    id: hash(SectionID::RevertStatus),
                    data: ItemData::Header(SectionHeader::Revert(revert.head)),
                    ..Default::default()
                }]
                .into_iter()
            } else {
                branch_status_items(&status.branch_status)?.into_iter()
            }
            .chain(if untracked.is_empty() {
                vec![]
            } else {
                vec![
                    items::blank_line(),
                    Item {
                        id: hash(SectionID::Untracked),
                        section: true,
                        depth: 0,
                        data: ItemData::AllUntracked(
                            untracked_files.iter().map(PathBuf::from).collect(),
                        ),
                        ..Default::default()
                    },
                ]
            })
            .chain(untracked)
            .chain(create_status_section_items(
                SectionID::UnstagedChanges,
                &Rc::new(git::diff_unstaged(repo.as_ref())?),
            ))
            .chain(create_status_section_items(
                SectionID::StagedChanges,
                &Rc::new(git::diff_staged(repo.as_ref())?),
            ))
            .chain(create_stash_list_section_items(
                repo.as_ref(),
                config.general.stash_list_limit,
            ))
            .chain(create_log_section_items(
                repo.as_ref(),
                config.general.recent_commits_limit,
            ))
            .collect();

            Ok(items)
        }),
    )
}

fn items_list(files: &[&String]) -> Vec<Item> {
    files
        .iter()
        .map(|path| Item {
            id: hash(path),
            depth: 1,
            data: ItemData::File(PathBuf::from(path)),
            ..Default::default()
        })
        .collect::<Vec<_>>()
}

fn branch_status_items(status: &BranchStatus) -> Res<Vec<Item>> {
    let Some(ref head) = status.local else {
        return Ok(vec![Item {
            id: hash(SectionID::BranchStatus),
            section: true,
            depth: 0,
            data: ItemData::Header(SectionHeader::NoBranch),
            ..Default::default()
        }]);
    };

    let mut items = vec![Item {
        id: hash(SectionID::BranchStatus),
        section: true,
        depth: 0,
        data: ItemData::Header(SectionHeader::OnBranch(head.clone())),
        ..Default::default()
    }];

    let Some(ref upstream_name) = status.remote else {
        return Ok(items);
    };

    items.push(Item {
        id: hash(SectionID::BranchStatus),
        depth: 1,
        unselectable: true,
        data: ItemData::BranchStatus(upstream_name.clone(), status.ahead, status.behind),
        ..Default::default()
    });

    Ok(items)
}

fn create_status_section_items<'a>(
    section: SectionID,
    diff: &'a Rc<Diff>,
) -> impl Iterator<Item = Item> + 'a {
    if diff.file_diffs.is_empty() {
        vec![]
    } else {
        let count = diff.file_diffs.len();
        let item_data = match section {
            SectionID::UnstagedChanges => ItemData::AllUnstaged(count),
            SectionID::StagedChanges => ItemData::AllStaged(count),
            _ => unreachable!("no other status section should be created"),
        };

        vec![
            items::blank_line(),
            Item {
                id: hash(section),
                section: true,
                depth: 0,
                data: item_data,
                ..Default::default()
            },
        ]
    }
    .into_iter()
    .chain(items::create_diff_items(diff, 1, true))
}

fn create_stash_list_section_items<'a>(
    repo: &Repository,
    limit: usize,
) -> impl Iterator<Item = Item> + 'a {
    let stashes = items::stash_list(repo, limit).unwrap();
    if stashes.is_empty() {
        vec![]
    } else {
        vec![
            items::blank_line(),
            Item {
                id: hash(SectionID::Stashes),
                section: true,
                depth: 0,
                data: ItemData::Header(SectionHeader::Stashes),
                ..Default::default()
            },
        ]
    }
    .into_iter()
    .chain(stashes)
}

fn create_log_section_items<'a>(
    repo: &Repository,
    limit: usize,
) -> impl Iterator<Item = Item> + 'a {
    [
        Item {
            depth: 0,
            unselectable: true,
            ..Default::default()
        },
        Item {
            id: hash(SectionID::RecentCommits),
            section: true,
            depth: 0,
            data: ItemData::Header(SectionHeader::RecentCommits),
            ..Default::default()
        },
    ]
    .into_iter()
    .chain(items::log(repo, limit, None, None).unwrap())
}
