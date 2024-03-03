use crate::config::Config;
use crate::git::diff::Delta;
use crate::git::diff::Diff;
use crate::git::diff::Hunk;
use crate::Res;
use git2::Oid;
use git2::Repository;
use ratatui::style::Style;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::text::Span;
use similar::Algorithm;
use similar::ChangeTag;
use similar::TextDiff;
use std::borrow::Cow;
use std::iter;
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
    File(String),
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
                    delta.new_file.clone()
                ),
                &config.color.file,
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
        display: Line::styled(hunk.header.clone(), &config.color.hunk_header),
        section: true,
        depth,
        target_data: Some(target_data),
        ..Default::default()
    })
    .chain(format_diff_hunk_items(&config, depth + 1, hunk))
}

fn format_diff_hunk_items(config: &Config, depth: usize, hunk: &Hunk) -> Vec<Item> {
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

    // TODO A lot of collect going on here (and inside format_changes too)
    format_changes(config, &changes.iter().collect::<Vec<_>>())
        .into_iter()
        .map(|line| Item {
            display: line,
            unselectable: true,
            depth,
            target_data: None,
            ..Default::default()
        })
        .collect()
}

fn format_changes(
    config: &Config,
    changes: &[&similar::InlineChange<'_, str>],
) -> Vec<Line<'static>> {
    let color = &config.color;
    let lines = changes
        .iter()
        .map(|change| {
            let style = match change.tag() {
                ChangeTag::Equal => Style::new(),
                ChangeTag::Delete => (&color.removed).into(),
                ChangeTag::Insert => (&color.added).into(),
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

pub(crate) fn log(
    config: &Config,
    repo: &Repository,
    limit: usize,
    reference: Option<String>,
) -> Res<Vec<Item>> {
    let color = &config.color;
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
                    let style: Style = if reference.is_remote() {
                        &color.remote
                    } else if reference.is_tag() {
                        &color.tag
                    } else {
                        &color.branch
                    }
                    .into();

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
                iter::once(Span::styled(short_id, &color.oid))
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
