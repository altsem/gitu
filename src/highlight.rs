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

    // TODO include function context in syntax highlights?
    let old_syntax_highlights =
        iter_syntax_highlights(&config.style.syntax_highlight, old_path, old_mask);
    let new_syntax_highlights =
        iter_syntax_highlights(&config.style.syntax_highlight, new_path, new_mask);
    let diff_highlights = iter_diff_highlights(&config.style.diff_highlight, &diff.text, hunk);

    let mut highlights_iter = zip_styles(
        zip_styles(old_syntax_highlights, new_syntax_highlights),
        diff_highlights,
    );

    iter_line_ranges(hunk_content).map(move |(line_range, line)| {
        let highlights = collect_line_highlights(&mut highlights_iter, &line_range);
        (line, highlights)
    })
}

pub(crate) fn iter_diff_highlights<'a>(
    config: &'a DiffHighlightConfig,
    source: &'a str,
    hunk: &'a gitu_diff::Hunk,
) -> Peekable<impl Iterator<Item = (Range<usize>, Style)> + 'a> {
    let hunk_content = source[hunk.content.range.clone()].as_bytes();

    fill_gaps(
        0..hunk_content.len(),
        hunk.content
            .changes
            .iter()
            .filter_map(|change| {
                let (Some(old_change), Some(new_change)) = (&change.old, &change.new) else {
                    return None;
                };

                let base = hunk.content.range.start;

                // TODO Might result in a lot of Vec allocations
                let (old, new): (Vec<_>, Vec<_>) = similar::capture_diff(
                    similar::Algorithm::Patience,
                    hunk_content,
                    (old_change.start - base)..(old_change.end - base),
                    hunk_content,
                    (new_change.start - base)..(new_change.end - base),
                )
                .into_iter()
                .map(|op| match op.tag() {
                    similar::DiffTag::Equal => (
                        Some((op.old_range(), Style::from(&config.unchanged_old))),
                        Some((op.new_range(), Style::from(&config.unchanged_new))),
                    ),
                    similar::DiffTag::Delete => (
                        Some((op.old_range(), Style::from(&config.changed_old))),
                        None,
                    ),
                    similar::DiffTag::Insert => (
                        None,
                        Some((op.new_range(), Style::from(&config.changed_new))),
                    ),
                    similar::DiffTag::Replace => (
                        Some((op.old_range(), Style::from(&config.changed_old))),
                        Some((op.new_range(), Style::from(&config.changed_new))),
                    ),
                })
                .unzip();

                Some(old.into_iter().chain(new).flatten())
            })
            .flatten(),
    )
    .peekable()
}

pub(crate) fn iter_line_ranges(content: &str) -> impl Iterator<Item = (Range<usize>, &str)> + '_ {
    content
        .split_inclusive('\n')
        .scan(0..0, |prev_line_range, line| {
            let line_range = prev_line_range.end..(prev_line_range.end + line.len());
            *prev_line_range = line_range.clone();
            Some((line_range, line))
        })
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
