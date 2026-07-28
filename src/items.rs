use crate::Res;
use crate::error::Error;
use crate::git::diff::Diff;
use crate::highlight;
use crate::item_data::ItemData;
use crate::item_data::Ref;
use git2::Oid;
use git2::Repository;
use regex::Regex;
use std::hash::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::iter;
use std::rc::Rc;
use std::time::SystemTime;

pub type ItemId = u64;

#[derive(Default, Clone, Debug)]
pub(crate) struct Item {
    pub(crate) id: ItemId,
    pub(crate) default_collapsed: bool,
    pub(crate) depth: usize,
    pub(crate) unselectable: bool,
    pub(crate) data: ItemData,
}

pub(crate) fn create_diff_items(
    diff: &Rc<Diff>,
    depth: usize,
    default_collapsed: bool,
    commit: Option<String>,
) -> impl Iterator<Item = Item> + '_ {
    diff.file_diffs
        .iter()
        .enumerate()
        .flat_map(move |(file_i, file_diff)| {
            iter::once(Item {
                id: hash(diff.file_diff_header(file_i)),
                default_collapsed,
                depth,
                data: ItemData::Delta {
                    diff: Rc::clone(diff),
                    file_i,
                    commit: commit.clone(),
                },
                ..Default::default()
            })
            .chain(file_diff.hunks.iter().cloned().enumerate().flat_map(
                move |(hunk_i, _hunk)| {
                    create_hunk_items(Rc::clone(diff), file_i, hunk_i, depth + 1)
                },
            ))
        })
}

fn create_hunk_items(
    diff: Rc<Diff>,
    file_i: usize,
    hunk_i: usize,
    depth: usize,
) -> impl Iterator<Item = Item> {
    let hunk_hash = hash([diff.file_diff_header(file_i), diff.hunk(file_i, hunk_i)]);
    iter::once(Item {
        id: hunk_hash,
        depth,
        data: ItemData::Hunk {
            diff: Rc::clone(&diff),
            file_i,
            hunk_i,
        },
        ..Default::default()
    })
    .chain(format_diff_hunk_items(
        diff,
        file_i,
        hunk_i,
        depth + 1,
        hunk_hash,
    ))
}

fn format_diff_hunk_items(
    diff: Rc<Diff>,
    file_i: usize,
    hunk_i: usize,
    depth: usize,
    hunk_hash: u64,
) -> Vec<Item> {
    let hunk_content = diff.hunk_content(file_i, hunk_i);

    highlight::line_range_iterator(hunk_content)
        .enumerate()
        .map(|(line_index, (line_range, line))| {
            Item {
                id: hunk_hash,
                // line is marked unselectable if it starts with a space character
                unselectable: line.starts_with(' '),
                depth,
                data: ItemData::HunkLine {
                    diff: Rc::clone(&diff),
                    file_i,
                    hunk_i,
                    line_i: line_index,
                    line_range,
                },
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
            let stash_ref = format!("stash@{{{}}}", i);
            Ok(Item {
                id: hash(stash_id),
                depth: 1,
                data: ItemData::Stash {
                    message: stash.message().unwrap_or("").to_string(),
                    stash_ref,
                    id: i,
                },
                ..Default::default()
            })
        })
        .map(|result| match result {
            Ok(item) => item,
            Err(err) => {
                let err = err.to_string();
                Item {
                    id: hash(&err),
                    data: ItemData::Error(err),
                    ..Default::default()
                }
            }
        })
        .take(limit)
        .collect::<Vec<_>>())
}

fn short_age(time: git2::Time) -> String {
    const MINUTE: i64 = 60;
    const HOUR: i64 = 60 * MINUTE;
    const DAY: i64 = 24 * HOUR;
    const WEEK: i64 = 7 * DAY;
    const MONTH: i64 = 30 * DAY;
    const YEAR: i64 = 365 * DAY;

    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_or(0, |since_epoch| since_epoch.as_secs() as i64);

    // A commit dated in the future reads as brand new, rather than as a negative age.
    let age = (now - time.seconds()).max(0);

    if age < HOUR {
        format!("{}m", age / MINUTE)
    } else if age < DAY {
        format!("{}h", age / HOUR)
    } else if age < WEEK {
        format!("{}d", age / DAY)
    } else if age < MONTH {
        format!("{}w", age / WEEK)
    } else if age < YEAR {
        format!("{}M", age / MONTH)
    } else {
        format!("{}y", age / YEAR)
    }
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
                        Ref::Remote(name)
                    } else if reference.is_tag() {
                        Ref::Tag(name)
                    } else {
                        Ref::Head(name)
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

            if let Some(re) = &msg_regex
                && !re.is_match(commit.message().unwrap_or(""))
            {
                return Ok(None);
            }

            let associated_references: Vec<_> = references
                .iter()
                .filter(|(commit, _)| commit.id() == oid)
                .map(|(_, reference)| reference.clone())
                .collect();

            let data = ItemData::Commit {
                oid: oid.to_string(),
                short_id,
                associated_references,
                summary: commit.summary().unwrap_or("").to_string(),
                author: commit.author().name().unwrap_or("").to_string(),
                age: short_age(commit.author().when()),
            };

            Ok(Some(Item {
                id: hash(oid),
                depth: 1,
                data,
                ..Default::default()
            }))
        })
        .filter_map(|result| match result {
            Ok(item) => item,
            Err(err) => {
                let err = err.to_string();
                Some(Item {
                    id: hash(&err),
                    data: ItemData::Error(err),
                    ..Default::default()
                })
            }
        })
        .take(limit)
        .collect();

    if items.is_empty() {
        Ok(vec![Item {
            data: ItemData::Raw("No commits found".to_string()),
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
        ..Default::default()
    }
}

pub(crate) fn hash<T: Hash>(x: T) -> ItemId {
    let mut hasher = DefaultHasher::new();
    x.hash(&mut hasher);
    hasher.finish()
}
