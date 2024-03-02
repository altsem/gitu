use crate::git::diff::Delta;
use crate::git::diff::Diff;
use crate::git::diff::Hunk;
use crate::theme;
use crate::theme::CURRENT_THEME;
use crate::Res;
use git2::Oid;
use git2::Repository;
use ratatui::style::Style;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::text::Text;
use similar::Algorithm;
use similar::ChangeTag;
use similar::TextDiff;
use std::borrow::Cow;
use std::iter;

#[derive(Default, Clone, Debug)]
pub(crate) struct Item {
    pub(crate) id: Cow<'static, str>,
    pub(crate) display: Text<'static>,
    pub(crate) section: bool,
    pub(crate) default_collapsed: bool,
    pub(crate) depth: usize,
    pub(crate) unselectable: bool,
    pub(crate) target_data: Option<TargetData>,
}

#[derive(Clone, Debug)]
pub(crate) enum TargetData {
    Commit(String),
    File(String),
    Delta(Delta),
    Hunk(Hunk),
    Branch(String),
}

pub(crate) fn create_diff_items<'a>(
    diff: &'a Diff,
    depth: &'a usize,
    default_collapsed: bool,
) -> impl Iterator<Item = Item> + 'a {
    diff.deltas.iter().flat_map(move |delta| {
        let target_data = TargetData::Delta(delta.clone());

        iter::once(Item {
            id: delta.file_header.to_string().into(),
            display: if delta.old_file == delta.new_file {
                delta.new_file.clone().fg(CURRENT_THEME.file)
            } else {
                format!("{} -> {}", delta.old_file, delta.new_file).fg(CURRENT_THEME.file)
            }
            .into(),
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
                .flat_map(|hunk| create_hunk_items(hunk, *depth)),
        )
    })
}

fn create_hunk_items(hunk: &Hunk, depth: usize) -> impl Iterator<Item = Item> {
    let target_data = TargetData::Hunk(hunk.clone());

    iter::once(Item {
        id: hunk.format_patch().into(),
        display: hunk
            .header
            .clone()
            .fg(theme::CURRENT_THEME.hunk_header)
            .into(),
        section: true,
        depth: depth + 1,
        target_data: Some(target_data),
        ..Default::default()
    })
    .chain([{
        Item {
            display: format_diff_hunk(hunk),
            unselectable: true,
            depth: depth + 2,
            target_data: None,
            ..Default::default()
        }
    }])
}

fn format_diff_hunk(hunk: &Hunk) -> Text<'static> {
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

    Text::from(format_changes(&changes.iter().collect::<Vec<_>>()))
}

fn format_changes(changes: &[&similar::InlineChange<'_, str>]) -> Vec<Line<'static>> {
    let lines = changes
        .iter()
        .map(|change| {
            let style = match change.tag() {
                ChangeTag::Equal => Style::new(),
                ChangeTag::Delete => Style::new().fg(CURRENT_THEME.removed),
                ChangeTag::Insert => Style::new().fg(CURRENT_THEME.added),
            };

            let prefix = match change.tag() {
                ChangeTag::Equal => " ",
                ChangeTag::Delete => "-",
                ChangeTag::Insert => "+",
            };

            let some_emph = change.iter_strings_lossy().any(|(emph, _value)| emph);

            Line::from(
                iter::once(Span::styled(prefix, style))
                    .chain(change.iter_strings_lossy().map(|(emph, value)| {
                        Span::styled(
                            value.to_string(),
                            if some_emph {
                                if emph {
                                    style.bold()
                                } else {
                                    style.dim()
                                }
                            } else {
                                style
                            },
                        )
                    }))
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();

    lines
}

pub(crate) fn log(repo: &Repository, limit: usize, reference: Option<String>) -> Res<Vec<Item>> {
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
            |reference| match (reference.target(), reference.shorthand()) {
                (Some(target), Some(name)) => {
                    let style = if reference.is_remote() {
                        CURRENT_THEME.remote
                    } else if reference.is_tag() {
                        CURRENT_THEME.tag
                    } else {
                        CURRENT_THEME.branch
                    };

                    Some((target, Span::styled(name.to_string(), style)))
                }
                _ => None,
            },
        )
        .collect::<Vec<(Oid, Span)>>();

    Ok(revwalk
        .map(|oid_result| -> Res<Item> {
            let oid = oid_result?;
            let commit = repo.find_commit(oid)?;
            let short_id = commit.as_object().short_id()?.as_str().unwrap().to_string();

            let spans = itertools::intersperse(
                iter::once(short_id.fg(CURRENT_THEME.oid))
                    .chain(
                        references
                            .iter()
                            .filter(|(ref_oid, _)| ref_oid == &oid)
                            .map(|(_, name)| name.clone()),
                    )
                    .chain([commit.summary().unwrap_or("").to_string().into()]),
                Span::raw(" "),
            )
            .collect::<Vec<_>>();

            Ok(Item {
                id: oid.to_string().into(),
                display: Line::from(spans).into(),
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
        display: Text::raw(""),
        depth: 0,
        unselectable: true,
        ..Default::default()
    }
}
