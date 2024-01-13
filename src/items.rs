use crate::diff;
use crate::process;
use diff::Delta;
use diff::Hunk;
use ratatui::style::Color;
use ratatui::style::Style;
use std::iter;

#[derive(Default, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Item {
    pub(crate) display: Option<(String, Style)>,
    pub(crate) section: bool,
    pub(crate) depth: usize,
    pub(crate) delta: Option<Delta>,
    pub(crate) hunk: Option<Hunk>,
    pub(crate) diff_line: Option<String>,
    pub(crate) reference: Option<String>,
}

pub(crate) fn create_diff_items(diff: diff::Diff, depth: usize) -> impl Iterator<Item = Item> {
    diff.deltas.into_iter().flat_map(move |delta| {
        let hunk_delta = delta.clone();

        iter::once(Item {
            delta: Some(delta.clone()),
            display: Some((
                if delta.old_file == delta.new_file {
                    delta.new_file
                } else {
                    format!("{} -> {}", delta.old_file, delta.new_file)
                },
                Style::new().fg(Color::Magenta),
            )),
            section: true,
            depth,
            ..Default::default()
        })
        .chain(
            delta
                .hunks
                .into_iter()
                .flat_map(move |hunk| create_hunk_items(&hunk, depth, &hunk_delta)),
        )
    })
}

fn create_hunk_items(hunk: &Hunk, depth: usize, hunk_delta: &Delta) -> impl Iterator<Item = Item> {
    iter::once(Item {
        display: Some((hunk.display_header(), Style::new().fg(Color::Blue))),
        section: true,
        depth: depth + 1,
        delta: Some(hunk_delta.clone()),
        hunk: Some(hunk.clone()),
        ..Default::default()
    })
    .chain([{
        let content = &format!("{}\n{}", hunk.header(), hunk.content);
        Item {
            display: Some((
                process::pipe(
                    content.as_bytes(),
                    &[
                        "delta",
                        &format!("-w {}", crossterm::terminal::size().unwrap().0),
                    ],
                )
                .0,
                Style::new(),
            )),
            depth: depth + 2,
            delta: Some(hunk_delta.clone()),
            hunk: Some(hunk.clone()),
            diff_line: Some("TODO".to_string()),
            ..Default::default()
        }
    }])
}

pub(crate) fn create_log_items(log: String) -> impl Iterator<Item = Item> {
    log.leak().lines().map(|log_line| Item {
        display: Some((log_line.to_string(), Style::new())),
        depth: 1,
        reference: Some(
            strip_ansi_escapes::strip_str(log_line)
                .to_string()
                .split_whitespace()
                .next()
                .expect("Error extracting ref")
                .to_string(),
        ),
        ..Default::default()
    })
}
