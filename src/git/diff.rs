use git2::Repository;
use itertools::Itertools;
use ratatui::{
    style::Style,
    text::{Line, Span, Text},
};
use similar::{udiff::UnifiedDiffHunk, Algorithm, ChangeTag, TextDiff};
use std::{fs, iter, ops::Range, path::PathBuf, rc::Rc, str};

use crate::{config::Config, Res};

#[derive(Debug, Clone)]
pub(crate) struct Diff {
    pub deltas: Vec<Delta>,
}

#[derive(Debug, Clone)]
pub(crate) struct Delta {
    pub file_header: String,
    pub old_file: PathBuf,
    pub new_file: PathBuf,
    pub hunks: Vec<Rc<Hunk>>,
    pub status: git2::Delta,
}

#[derive(Debug, Clone)]
pub(crate) struct Hunk {
    pub file_header: String,
    pub new_file: PathBuf,
    pub new_start: u32,
    pub header: String,
    pub content: Text<'static>,
}

#[derive(Debug)]
pub(crate) enum PatchMode {
    Normal,
    Reverse,
}

impl Hunk {
    pub(crate) fn format_patch(&self) -> String {
        format!("{}{}\n{}\n", &self.file_header, self.header, self.content)
    }

    pub(crate) fn format_line_patch(&self, line_range: Range<usize>, mode: PatchMode) -> String {
        let modified_content = self
            .content
            .lines
            .iter()
            .enumerate()
            .filter_map(|(i, line)| {
                let add = match mode {
                    PatchMode::Normal => '+',
                    PatchMode::Reverse => '-',
                };

                let remove = match mode {
                    PatchMode::Normal => '-',
                    PatchMode::Reverse => '+',
                };

                let patch_line = format!("{line}");

                if line_range.contains(&i) {
                    Some(patch_line)
                } else if patch_line.starts_with(add) {
                    None
                } else if let Some(stripped) = patch_line.strip_prefix(remove) {
                    Some(format!(" {}", stripped))
                } else {
                    Some(patch_line)
                }
            })
            .join("\n");

        format!(
            "{}{}\n{}\n",
            &self.file_header, self.header, modified_content
        )
    }

    pub(crate) fn first_diff_line(&self) -> u32 {
        self.content
            .lines
            .iter()
            .enumerate()
            .filter(|(_, line)| {
                let start = &line.spans.first().unwrap().content;
                start.starts_with('+') || start.starts_with('-')
            })
            .map(|(i, _)| i)
            .next()
            .unwrap_or(0) as u32
            + self.new_start
    }
}

pub(crate) fn convert_diff(
    config: &Config,
    repo: &Repository,
    diff: git2::Diff,
    workdir: bool,
) -> Res<Diff> {
    let mut deltas = vec![];

    diff.print(git2::DiffFormat::PatchHeader, |delta, _maybe_hunk, line| {
        let line_content = str::from_utf8(line.content()).unwrap();
        let is_new_header = line_content.starts_with("diff")
            && line.origin_value() == git2::DiffLineType::FileHeader;

        if is_new_header {
            let old_content = read_blob(repo, &delta.old_file());
            let new_content = if workdir {
                read_workdir(repo, &delta.new_file())
            } else {
                read_blob(repo, &delta.new_file())
            };

            let mut delta = Delta {
                file_header: line_content.to_string(),
                old_file: path(&delta.old_file()),
                new_file: path(&delta.new_file()),
                hunks: vec![],
                status: delta.status(),
            };

            delta.hunks = diff_files(config, &delta, old_content, new_content).unwrap();
            deltas.push(delta);
        } else {
            let delta = deltas.last_mut().unwrap();
            delta.file_header.push_str(line_content);
        }

        true
    })?;

    Ok(Diff { deltas })
}

fn diff_files(
    config: &Config,
    delta: &Delta,
    old_content: String,
    new_content: String,
) -> Res<Vec<Rc<Hunk>>> {
    let text_diff = TextDiff::configure()
        .algorithm(Algorithm::Patience)
        .diff_lines(&old_content, &new_content);

    Ok(text_diff
        .unified_diff()
        .iter_hunks()
        .map(|hunk| {
            let formatted_hunk = format_hunk(config, &hunk, &text_diff);

            let new_start = hunk
                .header()
                .to_string()
                .strip_prefix("@@ -")
                .unwrap()
                .split(|c| c == ' ' || c == ',')
                .next()
                .unwrap()
                .parse()
                .unwrap();

            Rc::new(Hunk {
                file_header: delta.file_header.clone(),
                new_file: delta.new_file.clone(),
                new_start,
                header: format!("{}", hunk.header()),
                content: formatted_hunk,
            })
        })
        .collect::<Vec<_>>())
}

fn format_hunk<'diff, 'old, 'new, 'bufs>(
    config: &Config,
    hunk: &UnifiedDiffHunk<'diff, 'old, 'new, 'bufs, str>,
    text_diff: &'diff TextDiff<'old, 'new, 'bufs, str>,
) -> Text<'static>
where
    'diff: 'old + 'new,
{
    let formatted_hunk = hunk.ops().iter().flat_map(|op| {
        text_diff
            .iter_inline_changes(op)
            .map(|change| format_line_change(config, &change))
    });

    formatted_hunk.collect::<Vec<_>>().into()
}

fn format_line_change(config: &Config, change: &similar::InlineChange<str>) -> Line<'static> {
    let style = &config.style;

    let line_style = match change.tag() {
        ChangeTag::Equal => Style::new(),
        ChangeTag::Delete => (&style.line_removed).into(),
        ChangeTag::Insert => (&style.line_added).into(),
    };

    let some_emph = change.iter_strings_lossy().any(|(emph, _value)| emph);

    let spans = iter::once(Span::styled(format!("{}", change.tag()), line_style))
        .chain(change.iter_strings_lossy().map(|(emph, value)| {
            Span::styled(
                value.trim_end_matches('\n').to_string(),
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
        .collect::<Vec<_>>();

    Line::from(spans)
}

fn read_workdir(repo: &Repository, new_file: &git2::DiffFile<'_>) -> String {
    fs::read_to_string(
        repo.workdir()
            .expect("No workdir")
            .join(new_file.path().unwrap()),
    )
    .unwrap()
}

fn read_blob(repo: &Repository, file: &git2::DiffFile<'_>) -> String {
    let blob = repo.find_blob(file.id());
    blob.map(|blob| String::from_utf8(blob.content().to_vec()).unwrap())
        .unwrap_or("".to_string())
}

fn path(file: &git2::DiffFile) -> PathBuf {
    file.path().unwrap().to_path_buf()
}
