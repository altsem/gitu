use crate::config::Config;
use crate::git::diff::Delta;
use crate::git::diff::Diff;
use crate::git::diff::Hunk;
use crate::Res;
use git2::Commit;
use git2::Oid;
use git2::Repository;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;
use regex::Regex;
use std::borrow::Cow;
use std::iter;
use std::path::PathBuf;
use std::rc::Rc;

#[derive(Default, Clone, Debug)]
pub(crate) struct Item {
    pub(crate) id: Cow<'static, str>,
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
    Delta(Delta),
    File(PathBuf),
    Hunk(Rc<Hunk>),
    HunkLine(Rc<Hunk>, usize),
    Stash { commit: String, id: usize },
}

pub(crate) fn create_diff_items<'a>(
    config: Rc<Config>,
    diff: &'a Diff,
    depth: &'a usize,
    default_collapsed: bool,
) -> impl Iterator<Item = Item> + 'a {
    diff.deltas.iter().flat_map(move |delta| {
        let target_data = TargetData::Delta(delta.clone());
        let config = Rc::clone(&config);

        iter::once(Item {
            id: delta.file_header.to_string().into(),
            display: Line::styled(
                format!(
                    "{}   {}",
                    format!("{:?}", delta.status).to_lowercase(),
                    delta.new_file.to_string_lossy()
                ),
                &config.style.file_header,
            ),
            section: true,
            default_collapsed,
            depth: *depth,
            target_data: Some(target_data),
            ..Default::default()
        })
        .chain(
            delta
                .hunks
                .iter()
                .cloned()
                .flat_map(move |hunk| create_hunk_items(Rc::clone(&config), hunk, *depth + 1)),
        )
    })
}

fn create_hunk_items(
    config: Rc<Config>,
    hunk: Rc<Hunk>,
    depth: usize,
) -> impl Iterator<Item = Item> {
    let target_data = TargetData::Hunk(Rc::clone(&hunk));

    iter::once(Item {
        id: hunk.format_patch().into(),
        display: Line::styled(hunk.header.clone(), &config.style.hunk_header),
        section: true,
        depth,
        target_data: Some(target_data),
        ..Default::default()
    })
    .chain(format_diff_hunk_items(depth + 1, hunk))
}

fn format_diff_hunk_items(depth: usize, hunk: Rc<Hunk>) -> Vec<Item> {
    hunk.content
        .lines
        .iter()
        .enumerate()
        .map(|(i, line)| Item {
            display: replace_tabs_with_spaces(line.clone()),
            unselectable: line
                .spans
                .first()
                .is_some_and(|s| s.content.starts_with(' ')),
            depth,
            target_data: Some(TargetData::HunkLine(Rc::clone(&hunk), i)),
            ..Default::default()
        })
        .collect()
}

fn replace_tabs_with_spaces(line: Line<'_>) -> Line<'_> {
    let spans = line
        .spans
        .iter()
        .cloned()
        .map(|span| Span::styled(span.content.replace('\t', "    "), span.style))
        .collect::<Vec<_>>();

    Line { spans, ..line }
}

pub(crate) fn stash_list(config: &Config, repo: &Repository, limit: usize) -> Res<Vec<Item>> {
    let style = &config.style;

    Ok(repo
        .reflog("refs/stash")?
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
                id: stash.id_new().to_string().into(),
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
                id: err.to_string().into(),
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
    let mut revwalk = repo.revwalk()?;
    if let Some(r) = rev {
        revwalk.push(r)?;
    } else if revwalk.push_head().is_err() {
        return Ok(vec![]);
    }

    let references = repo
        .references()?
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
            let oid = oid_result?;
            let commit = repo.find_commit(oid)?;
            let short_id = commit.as_object().short_id()?.as_str().unwrap().to_string();

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
                id: oid.to_string().into(),
                display: Line::from(spans),
                depth: 1,
                target_data: Some(TargetData::Commit(oid.to_string())),
                ..Default::default()
            }))
        })
        .filter_map(|result| match result {
            Ok(item) => item,
            Err(err) => Some(Item {
                id: err.to_string().into(),
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
