use crate::config::Config;
use crate::config::DiffHighlightConfig;
use crate::config::SyntaxHighlightConfig;
use crate::git::diff::Diff;
use crate::syntax_parser;
use crate::syntax_parser::SyntaxTag;
use itertools::Itertools;
use ratatui::style::Style;
use std::iter;
use std::iter::Peekable;
use std::ops::Range;
use std::path::Path;
use std::rc::Rc;
use unicode_segmentation::UnicodeSegmentation;

type LineHighlights<'a> = (&'a str, Vec<(Range<usize>, Style)>);

pub(crate) fn highlight_hunk_lines<'a>(
    config: &'a Config,
    diff: &'a Rc<Diff>,
    file_i: usize,
    hunk_i: usize,
) -> impl Iterator<Item = LineHighlights<'a>> + 'a {
    let old_path = &diff.text[diff.file_diffs[file_i].header.old_file.clone()];
    let new_path = &diff.text[diff.file_diffs[file_i].header.new_file.clone()];

    let hunk = &diff.file_diffs[file_i].hunks[hunk_i];
    let hunk_content = &diff.text[hunk.content.range.clone()];
    let old_mask = diff.mask_old_hunk(file_i, hunk_i);
    let new_mask = diff.mask_new_hunk(file_i, hunk_i);

    let old_syntax_highlights =
        iter_syntax_highlights(&config.style.syntax_highlight, old_path, old_mask);
    let new_syntax_highlights =
        iter_syntax_highlights(&config.style.syntax_highlight, new_path, new_mask);
    let diff_highlights = iter_diff_highlights(&config.style.diff_highlight, hunk_content, hunk);

    let mut highlights_iter = zip_styles(
        zip_styles(old_syntax_highlights, new_syntax_highlights),
        diff_highlights,
    );

    hunk_content
        .split_inclusive('\n')
        .scan_byte_ranges()
        .map(move |(line_range, line)| {
            let highlights = collect_line_highlights(&mut highlights_iter, &line_range);
            (line, highlights)
        })
}

pub(crate) fn iter_diff_highlights<'a>(
    config: &'a DiffHighlightConfig,
    hunk_text: &'a str,
    hunk: &'a gitu_diff::Hunk,
) -> Peekable<impl Iterator<Item = (Range<usize>, Style)> + 'a> {
    let hunk_bytes = hunk_text.as_bytes();

    fill_gaps(
        0..hunk_bytes.len(),
        hunk.content.changes.iter().flat_map(|change| {
            log::debug!("{change:?}");

            let base = hunk.content.range.start;

            let (old_indices, old_tokens): (Vec<_>, Vec<_>) = hunk_text
                [(change.old.start - base)..(change.old.end - base)]
                .split_word_bound_indices()
                .map(|(index, content)| (index + change.old.start - base, content))
                .unzip();

            let (new_indices, new_tokens): (Vec<_>, Vec<_>) = hunk_text
                [(change.new.start - base)..(change.new.end - base)]
                .split_word_bound_indices()
                .map(|(index, content)| (index + change.new.start - base, content))
                .unzip();

            let (old, new): (Vec<_>, Vec<_>) = similar::capture_diff(
                similar::Algorithm::Patience,
                &old_tokens,
                0..old_tokens.len(),
                &new_tokens,
                0..new_tokens.len(),
            )
            .into_iter()
            .map(|op| {
                let old_range = {
                    if old_indices.is_empty() {
                        change.old.clone()
                    } else {
                        let old_start = old_indices[op.old_range().start];
                        let old_end =
                            old_start + op.old_range().map(|i| old_tokens[i].len()).sum::<usize>();
                        old_start..old_end
                    }
                };

                let new_range = {
                    if new_indices.is_empty() {
                        change.new.clone()
                    } else {
                        let new_start = new_indices[op.new_range().start];
                        let new_end =
                            new_start + op.new_range().map(|i| new_tokens[i].len()).sum::<usize>();
                        new_start..new_end
                    }
                };

                let (old_style_config, new_style_config) = match op.tag() {
                    similar::DiffTag::Equal => (&config.unchanged_old, &config.unchanged_new),
                    _ => (&config.changed_old, &config.changed_new),
                };

                (
                    (old_range, Style::from(old_style_config)),
                    (new_range, Style::from(new_style_config)),
                )
            })
            .unzip();

            old.into_iter()
                .chain(new)
                .filter(|(range, _)| !range.is_empty())
        }),
    )
    .peekable()
}

trait ScanByteRanges<T> {
    fn scan_byte_ranges(self) -> impl Iterator<Item = (Range<usize>, T)>;
}

impl<'a, I: Iterator<Item = &'a str>> ScanByteRanges<&'a str> for I {
    fn scan_byte_ranges(self) -> impl Iterator<Item = (Range<usize>, &'a str)> {
        self.scan(0..0, |prev_line_range, line| {
            let line_range = prev_line_range.end..(prev_line_range.end + line.len());
            *prev_line_range = line_range.clone();
            Some((line_range, line))
        })
    }
}

pub(crate) fn iter_syntax_highlights<'a>(
    config: &'a SyntaxHighlightConfig,
    path: &'a str,
    content: String,
) -> Peekable<impl Iterator<Item = (Range<usize>, Style)> + 'a> {
    fill_gaps(
        0..content.len(),
        syntax_parser::parse(Path::new(path), &content)
            .into_iter()
            .map(move |(range, tag)| (range, syntax_highlight_tag_style(config, tag))),
    )
    .peekable()
}

pub(crate) fn fill_gaps<T: Clone + Default>(
    full_range: Range<usize>,
    ranges: impl Iterator<Item = (Range<usize>, T)>,
) -> impl Iterator<Item = (Range<usize>, T)> {
    iter::once((full_range.start, None))
        .chain(ranges.flat_map(|(range, item)| vec![(range.start, Some(item)), (range.end, None)]))
        .chain([(full_range.end, None)])
        .tuple_windows()
        .map(|((start, item_a), (end, _))| (start..end, item_a.unwrap_or_default()))
        .filter(|(range, _)| !range.is_empty())
        .peekable()
}

pub(crate) fn zip_styles(
    mut a: Peekable<impl Iterator<Item = (Range<usize>, Style)>>,
    mut b: Peekable<impl Iterator<Item = (Range<usize>, Style)>>,
) -> Peekable<impl Iterator<Item = (Range<usize>, Style)>> {
    iter::from_fn(move || next_merged_style(&mut a, &mut b))
        .dedup()
        .peekable()
}

/// Merges overlapping style-ranges from two iterators.
/// This should produce a continuous range, given that a and b are continuous.
pub(crate) fn next_merged_style(
    a: &mut Peekable<impl Iterator<Item = (Range<usize>, Style)>>,
    b: &mut Peekable<impl Iterator<Item = (Range<usize>, Style)>>,
) -> Option<(Range<usize>, Style)> {
    let ((a_range, a_style), (b_range, b_style)) = match (a.peek(), b.peek()) {
        (Some(a), Some(b)) => (a, b),
        (Some(_), None) => {
            return a.next();
        }
        (None, Some(_)) => {
            return b.next();
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
        Some(next)
    } else if a_range.contains(&b_range.start) {
        if a_range.contains(&(b_range.end - 1)) {
            // a: (       )
            // b:   ( X )
            let next = (b_range.start..b_range.end, a_style.patch(*b_style));
            b.next();
            Some(next)
        } else {
            // a: ( X )
            // b:   (   )
            let next = (b_range.start..a_range.end, a_style.patch(*b_style));
            a.next();
            Some(next)
        }
    } else if b_range.contains(&a_range.start) {
        if b_range.contains(&(a_range.end - 1)) {
            // a:   ( X )
            // b: (       )
            let next = (a_range.start..a_range.end, a_style.patch(*b_style));
            a.next();
            Some(next)
        } else {
            // a:   (   )
            // b: ( X )
            let next = (a_range.start..b_range.end, a_style.patch(*b_style));
            b.next();
            Some(next)
        }
    } else {
        unreachable!("ranges are disjoint: a: {:?} b: {:?}", a_range, b_range);
    }
}

pub(crate) fn collect_line_highlights(
    highlights_iter: &mut Peekable<impl Iterator<Item = (Range<usize>, Style)>>,
    line_range: &Range<usize>,
) -> Vec<(Range<usize>, Style)> {
    let mut spans = vec![];

    while let Some((range, style)) = highlights_iter.peek() {
        if range.end <= line_range.start {
            highlights_iter.next();
            continue;
        }

        let start = range.start.max(line_range.start);
        let end = range.end.min(line_range.end);

        spans.push(((start - line_range.start)..(end - line_range.start), *style));

        if line_range.end <= range.end {
            break;
        }

        highlights_iter.next();
    }

    spans
}

pub(crate) fn syntax_highlight_tag_style(config: &SyntaxHighlightConfig, tag: SyntaxTag) -> Style {
    match tag {
        SyntaxTag::Attribute => &config.attribute,
        SyntaxTag::Comment => &config.comment,
        SyntaxTag::Constant => &config.constant,
        SyntaxTag::ConstantBuiltin => &config.constant_builtin,
        SyntaxTag::Constructor => &config.constructor,
        SyntaxTag::Embedded => &config.embedded,
        SyntaxTag::Function => &config.function,
        SyntaxTag::FunctionBuiltin => &config.function_builtin,
        SyntaxTag::Keyword => &config.keyword,
        SyntaxTag::Module => &config.module,
        SyntaxTag::Number => &config.number,
        SyntaxTag::Operator => &config.operator,
        SyntaxTag::Property => &config.property,
        SyntaxTag::PunctuationBracket => &config.punctuation_bracket,
        SyntaxTag::PunctuationDelimiter => &config.punctuation_delimiter,
        SyntaxTag::String => &config.string,
        SyntaxTag::StringSpecial => &config.string_special,
        SyntaxTag::Tag => &config.tag,
        SyntaxTag::TypeBuiltin => &config.type_builtin,
        SyntaxTag::TypeRegular => &config.type_regular,
        SyntaxTag::VariableBuiltin => &config.variable_builtin,
        SyntaxTag::VariableParameter => &config.variable_parameter,
    }
    .into()
}
