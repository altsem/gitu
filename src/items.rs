use crate::git::diff::Delta;
use crate::git::diff::Diff;
use crate::git::diff::Hunk;
use crate::theme;
use crate::theme::CURRENT_THEME;
use ansi_to_tui::IntoText;
use ratatui::style::Color;
use ratatui::style::Modifier;
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

#[derive(Default, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Item {
    pub(crate) id: Cow<'static, str>,
    pub(crate) display: Text<'static>,
    pub(crate) section: bool,
    pub(crate) depth: usize,
    pub(crate) unselectable: bool,
    pub(crate) target_data: Option<TargetData>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum TargetData {
    Ref(String),
    File(String),
    Delta(Delta),
    Hunk(Hunk),
}

pub(crate) fn create_diff_items<'a>(
    diff: &'a Diff,
    depth: &'a usize,
) -> impl Iterator<Item = Item> + 'a {
    diff.deltas.iter().flat_map(|delta| {
        let target_data = TargetData::Delta(delta.clone());

        iter::once(Item {
            id: delta.file_header.to_string().into(),
            display: Text::styled(
                if delta.old_file == delta.new_file {
                    delta.new_file.clone()
                } else {
                    format!("{} -> {}", delta.old_file, delta.new_file)
                },
                Style::new().fg(CURRENT_THEME.file).bold(),
            ),
            section: true,
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
        display: Text::styled(
            hunk.header(),
            Style::new().fg(theme::CURRENT_THEME.hunk_header),
        ),
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

    let lines = diff
        .grouped_ops(4)
        .iter()
        .flat_map(|group| {
            group.iter().flat_map(|op| {
                diff.iter_inline_changes(op).map(|change| {
                    let color = match change.tag() {
                        ChangeTag::Equal => Color::Reset,
                        ChangeTag::Delete => CURRENT_THEME.removed,
                        ChangeTag::Insert => CURRENT_THEME.added,
                    };

                    Line::from(
                        change
                            .values()
                            .iter()
                            .map(|(emph, value)| {
                                Span::styled(
                                    value.to_string(),
                                    if *emph {
                                        Style::new().fg(color).add_modifier(Modifier::REVERSED)
                                    } else {
                                        Style::new().fg(color)
                                    },
                                )
                            })
                            .collect::<Vec<_>>(),
                    )
                })
            })
        })
        .collect::<Vec<_>>();

    Text::from(lines)
}

pub(crate) fn create_log_items(log: &str) -> impl Iterator<Item = Item> + '_ {
    log.lines().map(|log_line| {
        let target_data = TargetData::Ref(strip_ansi_escapes::strip_str(
            log_line
                .split_whitespace()
                .next()
                .expect("Error extracting ref"),
        ));

        Item {
            id: log_line.to_string().into(),
            display: log_line
                .to_string()
                .into_text()
                .expect("Error creating log text"),
            depth: 1,
            target_data: Some(target_data),
            ..Default::default()
        }
    })
}
