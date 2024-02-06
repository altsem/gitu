use crate::diff;
use crate::theme;
use crate::Config;
use ansi_to_tui::IntoText;
use diff::Delta;
use diff::Hunk;
use ratatui::style::Style;
use ratatui::text::Text;
use std::borrow::Cow;
use std::io::Write;
use std::iter;
use std::process::Command;
use std::process::Stdio;

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
    config: &'a Config,
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
            target_data: Some(target_data),
            ..Default::default()
        })
        .chain(
            delta
                .hunks
                .iter()
                .flat_map(|hunk| create_hunk_items(config, hunk, *depth)),
        )
    })
}

fn create_hunk_items(config: &Config, hunk: &Hunk, depth: usize) -> impl Iterator<Item = Item> {
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
            display: format_diff_hunk(config, hunk)
                .into_text()
                .expect("Error creating hunk text"),
            unselectable: true,
            depth: depth + 2,
            target_data: None,
            ..Default::default()
        }
    }])
}

fn format_diff_hunk(config: &Config, hunk: &Hunk) -> String {
    if config.use_delta {
        let content = format!("{}\n{}", hunk.header(), hunk.content);
        {
            let input = content.as_bytes();
            let cmd: &[&str] = &[
                "delta",
                "--color-only",
                &format!("-w {}", crossterm::terminal::size().unwrap().0),
            ];
            let mut command = Command::new(cmd[0])
                .args(&cmd[1..])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .expect("Couldn't spawn process");

            command
                .stdin
                .take()
                .unwrap_or_else(|| panic!("No stdin for {:?} process", cmd))
                .write_all(input)
                .expect("Couldn't write to process stdin");
            let output = command
                .wait_with_output()
                .unwrap_or_else(|_| panic!("Error writing {:?} output", cmd));

            String::from_utf8(output.stdout).unwrap()
        }
        .lines()
        .skip(1) // Header is already shown
        .collect::<Vec<_>>()
        .join("\n")
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
            target_data: Some(target_data),
            ..Default::default()
        }
    })
}
