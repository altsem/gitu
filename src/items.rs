use crate::config::Config;
use crate::git::diff::Delta;
use crate::git::diff::Diff;
use crate::git::diff::Hunk;
use crate::Res;
use git2::Commit;
use git2::Repository;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;
use similar::Algorithm;
use similar::ChangeTag;
use similar::TextDiff;
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
    Commit(String),
    File(PathBuf),
    Delta(Delta),
    Hunk(Hunk),
    Branch(String),
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
                .flat_map(move |hunk| create_hunk_items(Rc::clone(&config), hunk, *depth + 1)),
        )
    })
}

fn create_hunk_items(config: Rc<Config>, hunk: &Hunk, depth: usize) -> impl Iterator<Item = Item> {
    let target_data = TargetData::Hunk(hunk.clone());

    iter::once(Item {
        id: hunk.format_patch().into(),
        display: Line::styled(hunk.header.clone(), &config.style.hunk_header),
        section: true,
        depth,
        target_data: Some(target_data),
        ..Default::default()
    })
    .chain(format_diff_hunk_items(&config, depth + 1, hunk))
}

fn format_diff_hunk_items(
    config: &Config,
    depth: usize,
    hunk: &Hunk,
) -> impl Iterator<Item = Item> {
    let old = hunk.old_content();
    let new = hunk.new_content();

    let diff = TextDiff::configure()
        .algorithm(Algorithm::Patience)
        .diff_lines(&old, &new);

    let changes = diff
        .ops()
        .iter()
        .flat_map(|op| diff.iter_inline_changes(op))
        .collect::<Vec<_>>();

    format_changes(config, &changes)
        .into_iter()
        .map(move |line| Item {
            display: line,
            unselectable: true,
            depth,
            target_data: None,
            ..Default::default()
        })
}

fn format_changes(
    config: &Config,
    changes: &[similar::InlineChange<'_, str>],
) -> Vec<Line<'static>> {
    let style = &config.style;
    let lines = changes
        .iter()
        .map(|change| {
            let line_style = match change.tag() {
                ChangeTag::Equal => Style::new(),
                ChangeTag::Delete => (&style.line_removed).into(),
                ChangeTag::Insert => (&style.line_added).into(),
            };

            let prefix = match change.tag() {
                ChangeTag::Equal => " ",
                ChangeTag::Delete => "-",
                ChangeTag::Insert => "+",
            };

            let some_emph = change.iter_strings_lossy().any(|(emph, _value)| emph);

            Line::from(
                iter::once(Span::styled(prefix, line_style))
                    .chain(change.iter_strings_lossy().map(|(emph, value)| {
                        Span::styled(
                            value.to_string(),
                            if some_emph {
                                if emph {
                                    line_style.patch(&style.line_highlight.changed)
                                } else {
                                    line_style.patch(&style.line_highlight.unchanged)
                                }
                            } else {
                                line_style
                            },
                        )
                    }))
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();

    lines
}

pub(crate) fn log(
    config: &Config,
    repo: &Repository,
    limit: usize,
    reference: Option<String>,
) -> Res<Vec<Item>> {
    let style = &config.style;
    let mut revwalk = repo.revwalk()?;
    if let Some(r) = reference {
        let oid = repo.revparse_single(&r)?.id();
        revwalk.push(oid)?;
    } else if revwalk.push_head().is_err() {
        return Ok(vec![]);
    }

    let references = repo
        .references()?
        .filter_map(Result::ok)
        .filter_map(
            |reference| match (reference.peel_to_commit(), reference.shorthand()) {
                (Ok(target), Some(name)) => {
                    if name.ends_with("/HEAD") {
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

    Ok(revwalk
        .map(|oid_result| -> Res<Item> {
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

            Ok(Item {
                id: oid.to_string().into(),
                display: Line::from(spans),
                depth: 1,
                target_data: Some(TargetData::Commit(oid.to_string())),
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
        .collect())
}

pub(crate) fn blank_line() -> Item {
    Item {
        display: Line::raw(""),
        depth: 0,
        unselectable: true,
        ..Default::default()
    }
}
