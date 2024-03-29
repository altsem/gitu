use git2::Repository;
use itertools::Itertools;
use ratatui::{
    style::Style,
    text::{Line, Span, Text},
};
use similar::{Algorithm, DiffTag, DiffableStr, TextDiff};
use std::{
    fs,
    iter::{self},
    ops::Range,
    path::PathBuf,
    rc::Rc,
    str,
};
use tree_sitter_highlight::Highlighter;

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

    diff.print(
        git2::DiffFormat::PatchHeader,
        |diffdelta, _maybe_hunk, line| {
            let line_content = str::from_utf8(line.content()).unwrap();
            let is_new_header = line_content.starts_with("diff")
                && line.origin_value() == git2::DiffLineType::FileHeader;

            if is_new_header {
                let mut delta = Delta {
                    file_header: line_content.to_string(),
                    old_file: path(&diffdelta.old_file()),
                    new_file: path(&diffdelta.new_file()),
                    hunks: vec![],
                    status: diffdelta.status(),
                };

                if let Ok(hunks) = diff_files(repo, diffdelta, workdir, config, &delta) {
                    delta.hunks = hunks;
                }

                deltas.push(delta);
            } else {
                let delta = deltas.last_mut().unwrap();
                delta.file_header.push_str(line_content);
            }

            true
        },
    )?;

    Ok(Diff { deltas })
}

fn diff_files(
    repo: &Repository,
    diffdelta: git2::DiffDelta<'_>,
    workdir: bool,
    config: &Config,
    delta: &Delta,
) -> Res<Vec<Rc<Hunk>>> {
    let old_content = read_blob(repo, &diffdelta.old_file())?;
    let new_content = if workdir {
        read_workdir(repo, &diffdelta.new_file())?
    } else {
        read_blob(repo, &diffdelta.new_file())?
    };

    diff_content(config, delta, &old_content, &new_content)
}

mod syntax_highlight {
    use crate::config::Config;

    use super::split_at_newlines;
    use ratatui::style::Style;
    use std::ops::Range;
    use tree_sitter_highlight::{Highlight, HighlightConfiguration, HighlightEvent, Highlighter};

    const HIGHLIGHT_NAMES: &[&str] = &[
        "attribute",
        "comment",
        "constant.builtin",
        "constant",
        "constructor",
        "embedded",
        "function.builtin",
        "function",
        "keyword",
        "number",
        "module",
        "property",
        "operator",
        "punctuation.bracket",
        "punctuation.delimiter",
        "string.special",
        "string",
        "tag",
        "type",
        "type.builtin",
        "variable.builtin",
        "variable.parameter",
    ];

    pub(crate) fn create_config() -> HighlightConfiguration {
        // TODO Add more languages, only Rust is used for now
        let mut rust_config = HighlightConfiguration::new(
            tree_sitter_rust::language(),
            tree_sitter_rust::HIGHLIGHT_QUERY,
            tree_sitter_rust::INJECTIONS_QUERY,
            "",
        )
        .unwrap();

        rust_config.configure(HIGHLIGHT_NAMES);
        rust_config
    }

    pub(crate) fn iter_highlights<'a>(
        config: &Config,
        highlighter: &'a mut Highlighter,
        syntax_highlight_config: &'a HighlightConfiguration,
        content: &'a str,
    ) -> impl Iterator<Item = (Range<usize>, Style)> + 'a {
        let style = &config.style;

        let styles = [
            (&style.syntax_highlight.attribute).into(),
            (&style.syntax_highlight.comment).into(),
            (&style.syntax_highlight.constant_builtin).into(),
            (&style.syntax_highlight.constant).into(),
            (&style.syntax_highlight.constructor).into(),
            (&style.syntax_highlight.embedded).into(),
            (&style.syntax_highlight.function_builtin).into(),
            (&style.syntax_highlight.function).into(),
            (&style.syntax_highlight.keyword).into(),
            (&style.syntax_highlight.number).into(),
            (&style.syntax_highlight.module).into(),
            (&style.syntax_highlight.property).into(),
            (&style.syntax_highlight.operator).into(),
            (&style.syntax_highlight.punctuation_bracket).into(),
            (&style.syntax_highlight.punctuation_delimiter).into(),
            (&style.syntax_highlight.string_special).into(),
            (&style.syntax_highlight.string).into(),
            (&style.syntax_highlight.tag).into(),
            (&style.syntax_highlight.type_regular).into(),
            (&style.syntax_highlight.type_builtin).into(),
            (&style.syntax_highlight.variable_builtin).into(),
            (&style.syntax_highlight.variable_parameter).into(),
        ];

        highlighter
            .highlight(syntax_highlight_config, content.as_bytes(), None, |_| None)
            .unwrap()
            .scan((0..0, Style::new()), move |current, event| {
                match event.unwrap() {
                    HighlightEvent::Source { start, end } => Some(Some((start..end, current.1))),
                    HighlightEvent::HighlightStart(Highlight(highlight)) => {
                        current.1 = styles[highlight];
                        Some(None)
                    }
                    HighlightEvent::HighlightEnd => {
                        current.1 = Style::new();
                        Some(None)
                    }
                }
            })
            .flatten()
            .flat_map(|style_range| split_at_newlines(content, style_range))
    }
}

fn split_at_newlines(
    content: &str,
    (range, style): (Range<usize>, Style),
) -> impl Iterator<Item = (Range<usize>, Style)> + '_ {
    let range_indices = iter::once(range.start)
        .chain(
            content[range.clone()]
                .match_indices('\n')
                .map(move |(i, _)| i + 1 + range.start),
        )
        .chain([range.end]);

    range_indices
        .tuple_windows()
        .map(move |(a, b)| (a..b, style))
}
fn diff_content(
    config: &Config,
    delta: &Delta,
    old_content: &str,
    new_content: &str,
) -> Res<Vec<Rc<Hunk>>> {
    let style = &config.style;
    let old_lines = old_content.tokenize_lines();
    let new_lines = new_content.tokenize_lines();

    let old_line_indices = byte_ranges(&old_lines);
    let new_line_indices = byte_ranges(&new_lines);

    let text_diff = TextDiff::configure()
        .algorithm(Algorithm::Patience)
        .diff_slices(&old_lines, &new_lines);

    let syntax_highlight_config = syntax_highlight::create_config();
    let old_highlighter = &mut Highlighter::new();
    let new_highlighter = &mut Highlighter::new();
    let old_syntax_highlights = &mut syntax_highlight::iter_highlights(
        config,
        old_highlighter,
        &syntax_highlight_config,
        old_content,
    )
    .peekable();

    let new_syntax_highlights = &mut syntax_highlight::iter_highlights(
        config,
        new_highlighter,
        &syntax_highlight_config,
        new_content,
    )
    .peekable();

    Ok(text_diff
        .unified_diff()
        .iter_hunks()
        .map(|hunk| {
            let mut lines = vec![];

            hunk.ops().iter().for_each(|op| {
                let (line_tag, old_line, new_line) = op.as_tag_tuple();

                let old_prefix = match line_tag {
                    DiffTag::Equal => Span::raw(" "),
                    _ => Span::styled("-", &style.diff_highlight.tag_old),
                };

                let old_lines_range = total_range(&old_line_indices[old_line.clone()]);
                let new_lines_range = total_range(&new_line_indices[new_line.clone()]);

                let old_words = old_content[old_lines_range.clone()].tokenize_unicode_words();
                let new_words = new_content[new_lines_range.clone()].tokenize_unicode_words();

                let old_word_indices = byte_ranges(&old_words);
                let new_word_indices = byte_ranges(&new_words);

                let word_diff = TextDiff::configure()
                    .algorithm(Algorithm::Myers)
                    .diff_slices(&old_words, &new_words);

                let word_diff_style_ranges = word_diff
                    .ops()
                    .iter()
                    .map(|word_op| {
                        let (word_tag, word_token_old, word_token_new) = word_op.as_tag_tuple();
                        let word_old = total_range(&old_word_indices[word_token_old]);
                        let word_new = total_range(&new_word_indices[word_token_new]);

                        (
                            word_tag,
                            (old_lines_range.start + word_old.start)
                                ..(old_lines_range.start + word_old.end),
                            (new_lines_range.start + word_new.start)
                                ..(new_lines_range.start + word_new.end),
                        )
                    })
                    .collect::<Vec<_>>();

                let mut old_diff_highlights = word_diff_style_ranges
                    .iter()
                    .filter(|(_, word_old, _)| !word_old.is_empty())
                    .map(|(word_tag, word_old, _)| {
                        let diff_style = match word_tag {
                            DiffTag::Equal => Style::from(&style.diff_highlight.unchanged_old),
                            DiffTag::Delete => Style::from(&style.diff_highlight.changed_old),
                            DiffTag::Insert => unreachable!(),
                            DiffTag::Replace => Style::from(&style.diff_highlight.changed_old),
                        };

                        (word_old.clone(), diff_style)
                    })
                    .flat_map(|style_range| split_at_newlines(old_content, style_range))
                    .peekable();

                let mut new_diff_highlights = word_diff_style_ranges
                    .iter()
                    .filter(|(_, _, word_new)| !word_new.is_empty())
                    .map(|(word_tag, _, word_new)| {
                        let diff_style = match word_tag {
                            DiffTag::Equal => Style::from(&style.diff_highlight.unchanged_new),
                            DiffTag::Delete => unreachable!(),
                            DiffTag::Insert => Style::from(&style.diff_highlight.changed_new),
                            DiffTag::Replace => Style::from(&style.diff_highlight.changed_new),
                        };

                        (word_new.clone(), diff_style)
                    })
                    .flat_map(|style_range| split_at_newlines(new_content, style_range))
                    .peekable();

                create_lines(
                    &old_line_indices[old_line.clone()],
                    old_syntax_highlights,
                    &mut old_diff_highlights,
                    old_prefix,
                    old_content,
                    &mut lines,
                );

                // Don't print both old/new if equal
                if line_tag != DiffTag::Equal {
                    let new_prefix = Span::styled("+", &style.diff_highlight.tag_new);

                    create_lines(
                        &new_line_indices[new_line.clone()],
                        new_syntax_highlights,
                        &mut new_diff_highlights,
                        new_prefix,
                        new_content,
                        &mut lines,
                    );
                }
            });

            let formatted_hunk = Text::from(lines);

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

fn byte_ranges(tokens: &[&str]) -> Vec<Range<usize>> {
    tokens
        .iter()
        .scan(0, |count, x| {
            let len = x.len();
            let start = *count;
            let end = start + len;
            *count = end;
            Some(start..end)
        })
        .collect::<Vec<_>>()
}

fn total_range(lines: &[Range<usize>]) -> Range<usize> {
    lines
        .last()
        .map(|last| lines[0].start..last.end)
        .unwrap_or(0..0)
}

fn create_lines(
    line_indices: &[Range<usize>],
    syntax_highlights: &mut iter::Peekable<impl Iterator<Item = (Range<usize>, Style)>>,
    diff_highlights: &mut iter::Peekable<impl Iterator<Item = (Range<usize>, Style)>>,
    prefix: Span<'static>,
    content: &str,
    lines: &mut Vec<Line<'_>>,
) {
    for line in line_indices {
        advance_to(syntax_highlights, line.start);
        advance_to(diff_highlights, line.start);

        let a = &mut syntax_highlights
            .peeking_take_while(|(h_range, _)| h_range.start < line.end)
            .peekable();

        let b = &mut diff_highlights
            .peeking_take_while(|(h_range, _)| h_range.start < line.end)
            .peekable();

        let spans = iter::once(prefix.clone())
            .chain(
                iter::from_fn(|| next_merged_style_range(a, b))
                    .flatten()
                    .map(|(h_range, h_style)| {
                        (
                            // clamp to line
                            line.start.max(h_range.start)..line.end.min(h_range.end),
                            h_style,
                        )
                    })
                    .map(|(h_range, h_style)| {
                        Span::styled(
                            content[h_range]
                                // TODO only need to do this for the last span
                                .trim_end_matches(|s| s == '\r' || s == '\n')
                                .to_string(),
                            h_style,
                        )
                    }),
            )
            .collect::<Vec<_>>();

        lines.push(Line::from(spans));
    }
}

fn advance_to(iter: &mut iter::Peekable<impl Iterator<Item = (Range<usize>, Style)>>, to: usize) {
    while let Some((range, _style)) = iter.peek() {
        if range.end <= to {
            iter.next();
        } else {
            break;
        }
    }
}

/// Merges overlapping style-ranges from two iterators.
/// This should produce a continuous range, given that a and b are continuous.
fn next_merged_style_range(
    a: &mut iter::Peekable<impl Iterator<Item = (Range<usize>, Style)>>,
    b: &mut iter::Peekable<impl Iterator<Item = (Range<usize>, Style)>>,
) -> Option<Option<(Range<usize>, Style)>> {
    let ((a_range, a_style), (b_range, b_style)) = match (a.peek(), b.peek()) {
        (Some(a), Some(b)) => (a, b),
        (Some(_), None) => {
            return a.next().map(Some);
        }
        (None, Some(_)) => {
            return b.next().map(Some);
        }
        (None, None) => {
            return None;
        }
    };

    if a_range.end == b_range.end {
        let next = (
            a_range.start.max(b_range.start)..a_range.end,
            a_style.patch(*b_style),
        );
        a.next();
        b.next();
        Some(Some(next))
    } else if a_range.contains(&b_range.start) {
        if a_range.contains(&(b_range.end - 1)) {
            // a: (       )
            // b:   ( X )
            let next = (b_range.start..b_range.end, a_style.patch(*b_style));
            b.next();
            Some(Some(next))
        } else {
            // a: ( X )
            // b:   (   )
            let next = (b_range.start..a_range.end, a_style.patch(*b_style));
            a.next();
            Some(Some(next))
        }
    } else if b_range.contains(&a_range.start) {
        if b_range.contains(&(a_range.end - 1)) {
            // a:   ( X )
            // b: (       )
            let next = (a_range.start..a_range.end, a_style.patch(*b_style));
            a.next();
            Some(Some(next))
        } else {
            // a:   (   )
            // b: ( X )
            let next = (a_range.start..b_range.end, a_style.patch(*b_style));
            b.next();
            Some(Some(next))
        }
    } else {
        unreachable!("ranges are disjoint: a: {:?} b: {:?}", a_range, b_range);
    }
}

fn read_workdir(repo: &Repository, new_file: &git2::DiffFile<'_>) -> Res<String> {
    Ok(fs::read_to_string(
        repo.workdir()
            .expect("No workdir")
            .join(new_file.path().unwrap()),
    )?)
}

fn read_blob(repo: &Repository, file: &git2::DiffFile<'_>) -> Res<String> {
    let blob = repo.find_blob(file.id());
    blob.map(|blob| Ok(String::from_utf8(blob.content().to_vec())?))
        .unwrap_or(Ok("".to_string()))
}

fn path(file: &git2::DiffFile) -> PathBuf {
    file.path().unwrap().to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::Delta;
    use crate::config;

    #[test]
    fn changed_line() {
        let hunks = diff_content("old line\n", "new line\n");
        insta::assert_snapshot!(hunks[0].format_patch());
    }

    #[test]
    fn multiple_changed_lines() {
        let hunks = diff_content("one\ntwo\nthree\n", "three\ntwo\none\n");
        insta::assert_snapshot!(hunks[0].format_patch());
    }

    fn diff_content(old_content: &str, new_content: &str) -> Vec<std::rc::Rc<super::Hunk>> {
        super::diff_content(
            &config::init_test_config().unwrap(),
            &Delta {
                file_header: "header\n".into(),
                new_file: "new_file".into(),
                old_file: "old_file".into(),
                hunks: vec![],
                status: git2::Delta::Modified,
            },
            old_content,
            new_content,
        )
        .unwrap()
    }
}
