use crate::config::Config;
use crate::error::Error;
use crate::git::diff::Diff;
use crate::gitu_diff;
use crate::highlight;
use crate::Res;
use git2::Commit;
use git2::Oid;
use git2::Repository;
use gitu_diff::Status;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;
use regex::Regex;
use std::hash::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::iter;
use std::path::PathBuf;
use std::rc::Rc;

pub type ItemId = u64;

#[derive(Default, Clone, Debug)]
pub(crate) struct Item {
    pub(crate) id: ItemId,
    pub(crate) display: Line<'static>,
    pub(crate) section: bool,
    pub(crate) default_collapsed: bool,
    pub(crate) depth: usize,
    pub(crate) unselectable: bool,
    pub(crate) target_data: Option<TargetData>,
}

#[derive(Clone, Debug)]
pub(crate) enum TargetData {
    AllStaged,
    AllUnstaged,
    AllUntracked(Vec<PathBuf>),
    Branch(String),
    Commit(String),
    File(PathBuf),
    Delta {
        diff: Rc<Diff>,
        file_i: usize,
    },
    Hunk {
        diff: Rc<Diff>,
        file_i: usize,
        hunk_i: usize,
    },
    HunkLine {
        diff: Rc<Diff>,
        file_i: usize,
        hunk_i: usize,
        line_i: usize,
    },
    Stash {
        commit: String,
        id: usize,
    },
}

pub(crate) fn create_diff_items(
    config: Rc<Config>,
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
            let config = Rc::clone(&config);

            iter::once(Item {
                id: hash(diff.file_diff_header(file_i)),
                display: Line::styled(
                    format!(
                        "{:8}   {}",
                        format!("{:?}", file_diff.header.status).to_lowercase(),
                        match file_diff.header.status {
                            Status::Renamed => format!(
                                "{} -> {}",
                                &Rc::clone(diff).text[file_diff.header.old_file.clone()],
                                &Rc::clone(diff).text[file_diff.header.new_file.clone()]
                            ),
                            _ =>
                                Rc::clone(diff).text[file_diff.header.new_file.clone()].to_string(),
                        }
                    ),
                    &config.style.file_header,
                ),
                section: true,
                default_collapsed,
                depth,
                target_data: Some(target_data),
                ..Default::default()
            })
            .chain(file_diff.hunks.iter().cloned().enumerate().flat_map(
                move |(hunk_i, _hunk)| {
                    create_hunk_items(
                        Rc::clone(&config),
                        Rc::clone(diff),
                        file_i,
                        hunk_i,
                        depth + 1,
                    )
                },
            ))
        })
}

fn create_hunk_items(
    config: Rc<Config>,
    diff: Rc<Diff>,
    file_i: usize,
    hunk_i: usize,
    depth: usize,
) -> impl Iterator<Item = Item> {
    iter::once(Item {
        // TODO Don't do this
        id: hash([diff.file_diff_header(file_i), diff.hunk(file_i, hunk_i)]),
        display: Line::styled(
            diff.text[diff.file_diffs[file_i].hunks[hunk_i].header.range.clone()].to_string(),
            &config.style.hunk_header,
        ),
        section: true,
        depth,
        target_data: Some(TargetData::Hunk {
            diff: Rc::clone(&diff),
            file_i,
            hunk_i,
        }),
        ..Default::default()
    })
    .chain(format_diff_hunk_items(
        &config,
        diff,
        file_i,
        hunk_i,
        depth + 1,
    ))
}

fn format_diff_hunk_items(
    config: &Config,
    diff: Rc<Diff>,
    file_i: usize,
    hunk_i: usize,
    depth: usize,
) -> Vec<Item> {
    highlight::highlight_hunk_lines(config, &diff, file_i, hunk_i)
        .enumerate()
        .map(|(line_i, (line, spans))| {
            let display = Line::from(
                spans
                    .into_iter()
                    .map(|(range, style)| {
                        Span::styled(
                            line[range.clone()]
                                .trim_end_matches("\r\n")
                                .replace('\t', "    "),
                            style,
                        )
                    })
                    .collect::<Vec<_>>(),
            );
            let unselectable = line.starts_with(' ');

            Item {
                display,
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

pub(crate) fn stash_list(config: &Config, repo: &Repository, limit: usize) -> Res<Vec<Item>> {
    let style = &config.style;

    Ok(repo
        .reflog("refs/stash")
        .map_err(Error::StashList)?
        .iter()
        .enumerate()
        .map(|(i, stash)| -> Res<Item> {
            let spans = itertools::intersperse(
                iter::once(Span::styled(format!("stash@{i}"), &style.hash)).chain([stash
                    .message()
                    .unwrap_or("")
                    .to_string()
                    .into()]),
                Span::raw(" "),
            )
            .collect::<Vec<_>>();

            Ok(Item {
                id: hash(stash.id_new()),
                display: Line::from(spans),
                depth: 1,
                target_data: Some(TargetData::Stash {
                    commit: stash.id_new().to_string(),
                    id: i,
                }),
                ..Default::default()
            })
        })
        .map(|result| match result {
            Ok(item) => item,
            Err(err) => Item {
                id: hash(err.to_string()),
                display: err.to_string().into(),
                ..Default::default()
            },
        })
        .take(limit)
        .collect::<Vec<_>>())
}

pub(crate) fn log(
    config: &Config,
    repo: &Repository,
    limit: usize,
    rev: Option<Oid>,
    msg_regex: Option<Regex>,
) -> Res<Vec<Item>> {
    let style = &config.style;
    let mut revwalk = repo.revwalk().map_err(Error::ReadLog)?;
    if let Some(r) = rev {
        revwalk.push(r).map_err(Error::ReadLog)?;
    } else if revwalk.push_head().is_err() {
        return Ok(vec![]);
    }

    let references = repo
        .references()
        .map_err(Error::ReadLog)?
        .filter_map(Result::ok)
        .filter_map(
            |reference| match (reference.peel_to_commit(), reference.shorthand()) {
                (Ok(target), Some(name)) => {
                    if name.ends_with("/HEAD") || name.starts_with("prefetch/remotes/") {
                        return None;
                    }

                    let style: Style = if reference.is_remote() {
                        &style.remote
                    } else if reference.is_tag() {
                        &style.tag
                    } else {
                        &style.branch
                    }
                    .into();

                    Some((target, Span::styled(name.to_string(), style)))
                }
                _ => None,
            },
        )
        .collect::<Vec<(Commit, Span)>>();

    let items: Vec<Item> = revwalk
        .map(|oid_result| -> Res<Option<Item>> {
            let oid = oid_result.map_err(Error::ReadLog)?;
            let commit = repo.find_commit(oid).map_err(Error::ReadLog)?;
            let short_id =
                String::from_utf8_lossy(&commit.as_object().short_id().map_err(Error::ReadOid)?)
                    .to_string();

            let spans = itertools::intersperse(
                iter::once(Span::styled(short_id, &style.hash))
                    .chain(
                        references
                            .iter()
                            .filter(|(commit, _)| commit.id() == oid)
                            .map(|(_, name)| name.clone()),
                    )
                    .chain([commit.summary().unwrap_or("").to_string().into()]),
                Span::raw(" "),
            )
            .collect::<Vec<_>>();

            if let Some(re) = &msg_regex {
                if !re.is_match(commit.message().unwrap_or("")) {
                    return Ok(None);
                }
            }

            Ok(Some(Item {
                id: hash(oid),
                display: Line::from(spans),
                depth: 1,
                target_data: Some(TargetData::Commit(oid.to_string())),
                ..Default::default()
            }))
        })
        .filter_map(|result| match result {
            Ok(item) => item,
            Err(err) => Some(Item {
                id: hash(err.to_string()),
                display: err.to_string().into(),
                ..Default::default()
            }),
        })
        .take(limit)
        .collect();

    if items.is_empty() {
        Ok(vec![Item {
            display: Line::raw("No commits found"),
            ..Default::default()
        }])
    } else {
        Ok(items)
    }
}

pub(crate) fn blank_line() -> Item {
    Item {
        display: Line::raw(""),
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
