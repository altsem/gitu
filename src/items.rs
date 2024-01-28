use crate::diff;
use crate::keybinds;
use crate::keybinds::Op;
use crate::keybinds::TransientOp;
use crate::list_target_ops;
use crate::process;
use crate::theme;
use ansi_to_tui::IntoText;
use diff::Delta;
use diff::Hunk;
use ratatui::style::Style;
use ratatui::text::Text;
use std::borrow::Cow;
use std::iter;

#[derive(Default, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Item {
    pub(crate) id: Cow<'static, str>,
    pub(crate) display: Text<'static>,
    pub(crate) section: bool,
    pub(crate) depth: usize,
    pub(crate) unselectable: bool,
    pub(crate) key_hint: Option<String>,
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
    diff: &'a diff::Diff,
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
                Style::new().fg(theme::CURRENT_THEME.file),
            ),
            section: true,
            depth: *depth,
            key_hint: Some(key_hint(&target_data)),
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
            hunk.display_header(),
            Style::new().fg(theme::CURRENT_THEME.hunk_header),
        ),
        section: true,
        depth: depth + 1,
        key_hint: Some(key_hint(&target_data)),
        target_data: Some(target_data),
        ..Default::default()
    })
    .chain([{
        Item {
            display: format_diff_hunk(hunk)
                .into_text()
                .expect("Error creating hunk text"),
            unselectable: true,
            depth: depth + 2,
            target_data: None,
            ..Default::default()
        }
    }])
}

fn key_hint(target_data: &TargetData) -> String {
    list_target_ops(target_data)
        .into_iter()
        .filter_map(|target_op| {
            keybinds::display_key(TransientOp::None, Op::Target(target_op))
                .map(|key| format!("{} {:?}", key, target_op))
        })
        .collect::<Vec<_>>()
        .join("  ")
}

fn format_diff_hunk(hunk: &Hunk) -> String {
    if *crate::USE_DELTA {
        let content = format!("{}\n{}", hunk.header(), hunk.content);
        process::pipe(
            content.as_bytes(),
            &[
                "delta",
                &format!("-w {}", crossterm::terminal::size().unwrap().0),
            ],
        )
        .0
    } else {
        hunk.content.clone()
    }
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
            key_hint: Some(key_hint(&target_data)),
            target_data: Some(target_data),
            ..Default::default()
        }
    })
}
