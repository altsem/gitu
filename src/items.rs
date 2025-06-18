use crate::config::Config;
use crate::error::Error;
use crate::git::diff::Diff;
use crate::highlight;
use crate::target_data::RefKind;
use crate::target_data::TargetData;
use crate::Res;
use git2::Oid;
use git2::Repository;
use regex::Regex;
use similar::DiffableStr;
use std::hash::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::iter;
use std::rc::Rc;

pub type ItemId = u64;

#[derive(Default, Clone, Debug)]
pub(crate) struct Item {
    pub(crate) id: ItemId,
    // TODO We'll want to move away from this `Line` struct
    // preferably we can store text and styling separately like in highlight.rs: `(Range<usize>, Style)`
    // and only apply them when rendering
    pub(crate) section: bool,
    pub(crate) default_collapsed: bool,
    pub(crate) depth: usize,
    pub(crate) unselectable: bool,
    // TODO rename? maybe item_data: Option<ItemData>
    // TODO does this have to be optional anymore?
    pub(crate) target_data: Option<TargetData>,
}
pub(crate) fn create_diff_items(
    diff: &Rc<Diff>,
    depth: usize,
    default_collapsed: bool,
) -> impl Iterator<Item = Item> + '_ {
    diff.file_diffs
        .iter()
        .enumerate()
        .flat_map(move |(file_i, file_diff)| {
            let target_data = TargetData::Delta {
                diff: Rc::clone(diff),
                file_i,
            };

            iter::once(Item {
                id: hash(diff.file_diff_header(file_i)),
                section: true,
                default_collapsed,
                depth,
                target_data: Some(target_data),
                ..Default::default()
            })
            .chain(file_diff.hunks.iter().cloned().enumerate().flat_map(
                move |(hunk_i, _hunk)| {
                    create_hunk_items(Rc::clone(diff), file_i, hunk_i, depth + 1)
                },
            ))
        })
}

fn create_hunk_items<'a>(
    diff: Rc<Diff>,
    file_i: usize,
    hunk_i: usize,
    depth: usize,
) -> impl Iterator<Item = Item> {
    iter::once(Item {
        id: hash([diff.file_diff_header(file_i), diff.hunk(file_i, hunk_i)]),
        section: true,
        depth,
        target_data: Some(TargetData::Hunk {
            diff: Rc::clone(&diff),
            file_i,
            hunk_i,
        }),
        ..Default::default()
    })
    .chain(format_diff_hunk_items(diff, file_i, hunk_i, depth + 1))
}

fn format_diff_hunk_items(diff: Rc<Diff>, file_i: usize, hunk_i: usize, depth: usize) -> Vec<Item> {
    diff.text
        .lines()
        .enumerate()
        .map(|(line_i, line)| {
            let unselectable = line.starts_with(' ');

            Item {
                unselectable,
                depth,
                target_data: Some(TargetData::HunkLine {
                    diff: Rc::clone(&diff),
                    file_i,
                    hunk_i,
                    line_i,
                }),
                ..Default::default()
            }
        })
        .collect()
}

pub(crate) fn stash_list(repo: &Repository, limit: usize) -> Res<Vec<Item>> {
    Ok(repo
        .reflog("refs/stash")
        .map_err(Error::StashList)?
        .iter()
        .enumerate()
        .map(|(i, stash)| -> Res<Item> {
            let stash_id = stash.id_new();
            Ok(Item {
                id: hash(&stash_id),
                depth: 1,
                target_data: Some(TargetData::Stash {
                    message: stash.message().unwrap_or("").to_string(),
                    commit: stash_id.to_string(),
                    id: i,
                }),
                ..Default::default()
            })
        })
        .map(|result| match result {
            Ok(item) => item,
            Err(err) => {
                let err = err.to_string();
                Item {
                    id: hash(&err),
                    target_data: Some(TargetData::Error(err)),
                    ..Default::default()
                }
            }
        })
        .take(limit)
        .collect::<Vec<_>>())
}

pub(crate) fn log(
    repo: &Repository,
    limit: usize,
    rev: Option<Oid>,
    msg_regex: Option<Regex>,
) -> Res<Vec<Item>> {
    let mut revwalk = repo.revwalk().map_err(Error::ReadLog)?;
    if let Some(r) = rev {
        revwalk.push(r).map_err(Error::ReadLog)?;
    } else if revwalk.push_head().is_err() {
        return Ok(vec![]);
    }

    let references: Vec<_> = repo
        .references()
        .map_err(Error::ReadLog)?
        .filter_map(Result::ok)
        .filter_map(
            |reference| match (reference.peel_to_commit(), reference.shorthand()) {
                (Ok(target), Some(name)) => {
                    if name.ends_with("/HEAD") || name.starts_with("prefetch/remotes/") {
                        return None;
                    }

                    let name = name.to_owned();

                    let ref_kind = if reference.is_remote() {
                        RefKind::Remote(name)
                    } else if reference.is_tag() {
                        RefKind::Tag(name)
                    } else {
                        RefKind::Branch(name)
                    };

                    Some((target, ref_kind))
                }
                _ => None,
            },
        )
        .collect();

    let items: Vec<Item> = revwalk
        .map(|oid_result| -> Res<Option<Item>> {
            let oid = oid_result.map_err(Error::ReadLog)?;
            let commit = repo.find_commit(oid).map_err(Error::ReadLog)?;

            let short_id = commit.as_object().short_id().map_err(Error::ReadOid)?;
            let short_id = String::from_utf8_lossy(&short_id).to_string();

            if let Some(re) = &msg_regex {
                if !re.is_match(commit.message().unwrap_or("")) {
                    return Ok(None);
                }
            }

            let associated_references: Vec<_> = references
                .iter()
                .filter(|(commit, _)| commit.id() == oid)
                .map(|(_, reference)| reference.clone())
                .collect();

            let target_data = TargetData::Commit {
                oid: oid.to_string(),
                short_id,
                associated_references,
                summary: commit.summary().unwrap_or("").to_string(),
            };

            Ok(Some(Item {
                id: hash(oid),
                depth: 1,
                target_data: Some(target_data),
                ..Default::default()
            }))
        })
        .filter_map(|result| match result {
            Ok(item) => item,
            Err(err) => {
                let err = err.to_string();
                Some(Item {
                    id: hash(&err),
                    target_data: Some(TargetData::Error(err)),
                    ..Default::default()
                })
            }
        })
        .take(limit)
        .collect();

    if items.is_empty() {
        Ok(vec![Item {
            ..Default::default()
        }])
    } else {
        Ok(items)
    }
}

pub(crate) fn blank_line() -> Item {
    Item {
        depth: 0,
        unselectable: true,
        target_data: Some(TargetData::Empty),
        ..Default::default()
    }
}

pub(crate) fn hash<T: Hash>(x: T) -> ItemId {
    let mut hasher = DefaultHasher::new();
    x.hash(&mut hasher);
    hasher.finish()
}
