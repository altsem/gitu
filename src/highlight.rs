use crate::config::Config;
use crate::config::DiffHighlightConfig;
use crate::config::SyntaxHighlightConfig;
use crate::git::diff::Diff;
use crate::gitu_diff;
use crate::syntax_parser;
use crate::syntax_parser::SyntaxTag;
use cached::{SizedCache, proc_macro::cached};
use itertools::Itertools;
use ratatui::style::Style;
use std::iter;
use std::iter::Peekable;
use std::ops::Range;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;
use unicode_segmentation::UnicodeSegmentation;

#[cached(
    ty = "SizedCache<u64, Arc<HunkHighlights>>",
    create = "{ SizedCache::with_size(200) }",
    convert = r#"{ _hunk_hash }"#
)]
pub(crate) fn highlight_hunk(
    _hunk_hash: u64,
    config: &Config,
    diff: &Rc<Diff>,
    file_index: usize,
    hunk_index: usize,
) -> Arc<HunkHighlights> {
    let file_diff = &diff.file_diffs[file_index];

    let hunk_content = diff.hunk_content(file_index, hunk_index);

    let old_mask = diff.mask_old_hunk(file_index, hunk_index);
    let old_file_range = file_diff.header.old_file.clone();
    let old_path = &old_file_range.fmt(&diff.text);

    let new_mask = diff.mask_new_hunk(file_index, hunk_index);
    let new_file_range = file_diff.header.new_file.clone();
    let new_path = &new_file_range.fmt(&diff.text);

    let old_syntax_highlights =
        iter_syntax_highlights(&config.style.syntax_highlight, old_path, old_mask);
    let new_syntax_highlights =
        iter_syntax_highlights(&config.style.syntax_highlight, new_path, new_mask);

    let hunk = &diff.file_diffs[file_index].hunks[hunk_index];
    let diff_highlights = iter_diff_highlights(&config.style.diff_highlight, hunk_content, hunk);
    let diff_context_highlights =
        iter_diff_context_highlights(&config.style.diff_highlight, hunk_content);

    let mut highlights_iterator = zip_styles(
        zip_styles(old_syntax_highlights, new_syntax_highlights),
        zip_styles(diff_highlights, diff_context_highlights),
    );

    let mut highlights = HunkHighlights {
        spans: vec![],
        line_index: vec![],
    };

    for (line_range, _) in line_range_iterator(hunk_content) {
        let start = highlights.spans.len();

        collect_line_highlights(&mut highlights_iterator, &line_range, &mut highlights.spans);
        highlights.line_index.push(start..highlights.spans.len());
    }

    Arc::new(highlights)
}

#[derive(Clone)]
pub struct HunkHighlights {
    spans: Vec<(Range<usize>, Style)>,
    line_index: Vec<Range<usize>>,
}

impl HunkHighlights {}

impl HunkHighlights {
    /// Get highlight segments for a given hunk line.
    pub fn get_line_highlights(&self, line: usize) -> &[(Range<usize>, Style)] {
        let line_range = &self.line_index[line];
        &self.spans[line_range.clone()]
    }
}

/// Construct a newline inclusive iterator over each line in a chunk of text.
pub fn line_range_iterator(content: &str) -> impl Iterator<Item = (Range<usize>, &str)> {
    content
        .split_inclusive('\n')
        .scan(0usize, |prev_line_end, current_line| {
            let line_start = *prev_line_end;

            let actual_line_length = current_line.len();

            let visual_line_length = if current_line.ends_with("\r\n") {
                actual_line_length - 2
            } else {
                actual_line_length - 1
            };

            let actual_line_end = line_start + actual_line_length;

            let visual_line_end = line_start + visual_line_length;

            *prev_line_end = actual_line_end;

            Some((line_start..visual_line_end, current_line))
        })
}

pub(crate) fn iter_diff_highlights<'a>(
    config: &'a DiffHighlightConfig,
    hunk_text: &'a str,
    hunk: &'a gitu_diff::Hunk,
) -> Peekable<impl Iterator<Item = (Range<usize>, Style)> + 'a> {
    let hunk_bytes = hunk_text.as_bytes();

    let change_highlights = hunk.content.changes.iter().flat_map(|change| {
        let base = hunk.content.range.start;
        let old_range = change.old.start - base..change.old.end - base;
        let new_range = change.new.start - base..change.new.end - base;

        let (old_indices, old_tokens): (Vec<_>, Vec<_>) = hunk_text[old_range.clone()]
            .split_word_bound_indices()
            .map(|(index, content)| (index + old_range.start, content))
            .unzip();

        let (new_indices, new_tokens): (Vec<_>, Vec<_>) = hunk_text[new_range.clone()]
            .split_word_bound_indices()
            .map(|(index, content)| (index + new_range.start, content))
            .unzip();

        let mut interner = imara_diff::Interner::new(old_tokens.len() + new_tokens.len());
        let old_token_ids: Vec<imara_diff::Token> = old_tokens
            .iter()
            .map(|&token| interner.intern(token))
            .collect();

        let new_token_ids: Vec<imara_diff::Token> = new_tokens
            .iter()
            .map(|&token| interner.intern(token))
            .collect();

        let mut diff = imara_diff::Diff::default();
        diff.compute_with(
            imara_diff::Algorithm::Histogram,
            &old_token_ids,
            &new_token_ids,
            interner.num_tokens(),
        );

        let mut old = Vec::new();
        let mut new = Vec::new();

        for hunk in diff.hunks() {
            if !hunk.before.is_empty() {
                let old_start = old_indices[hunk.before.start as usize];
                let old_end = old_indices[hunk.before.end as usize - 1]
                    + old_tokens[hunk.before.end as usize - 1].len();
                old.push((old_start..old_end, Style::from(&config.changed_old)));
            }
        }

        for hunk in diff.hunks() {
            if !hunk.after.is_empty() {
                let new_start = new_indices[hunk.after.start as usize];
                let new_end = new_indices[hunk.after.end as usize - 1]
                    + new_tokens[hunk.after.end as usize - 1].len();
                new.push((new_start..new_end, Style::from(&config.changed_new)));
            }
        }

        old.into_iter()
            .chain(new)
            .filter(|(range, _)| !range.is_empty())
    });

    fill_gaps(
        0..hunk_bytes.len(),
        change_highlights,
        Style::from(&config.unchanged_old),
    )
    .peekable()
}

pub(crate) fn iter_diff_context_highlights<'a>(
    config: &'a DiffHighlightConfig,
    hunk_text: &'a str,
) -> Peekable<impl Iterator<Item = (Range<usize>, Style)> + 'a> {
    fill_gaps(
        0..hunk_text.len(),
        line_range_iterator(hunk_text).flat_map(|(range, line)| {
            if line.starts_with('-') {
                Some((range.start..range.start + 1, Style::from(&config.tag_old)))
            } else if line.starts_with('+') {
                Some((range.start..range.start + 1, Style::from(&config.tag_new)))
            } else {
                None
            }
        }),
        Style::new(),
    )
    .peekable()
}

pub(crate) fn iter_syntax_highlights<'a>(
    config: &'a SyntaxHighlightConfig,
    path: &'a str,
    content: String,
) -> Peekable<impl Iterator<Item = (Range<usize>, Style)> + 'a> {
    let highlights = if config.enabled {
        syntax_parser::parse(Path::new(path), &content)
    } else {
        Vec::new()
    };

    fill_gaps(
        0..content.len(),
        highlights
            .into_iter()
            .map(move |(range, tag)| (range, syntax_highlight_tag_style(config, tag))),
        Style::new(),
    )
    .peekable()
}

pub(crate) fn fill_gaps<T: Clone + Default>(
    full_range: Range<usize>,
    ranges: impl Iterator<Item = (Range<usize>, T)>,
    fill: T,
) -> impl Iterator<Item = (Range<usize>, T)> {
    iter::once((full_range.start, None))
        .chain(ranges.flat_map(|(range, item)| vec![(range.start, Some(item)), (range.end, None)]))
        .chain([(full_range.end, None)])
        .tuple_windows()
        .map(move |((start, item_a), (end, _))| (start..end, item_a.unwrap_or(fill.clone())))
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
    result: &mut Vec<(Range<usize>, Style)>,
) {
    while let Some((range, style)) = highlights_iter.peek() {
        // if the current range in the iter ends before the line we
        // are interested in highlights for, we advance the iterator
        // and continue the loop
        if range.end <= line_range.start {
            highlights_iter.next();
            continue;
        }

        // clamp the range to within the given line range
        let start = range.start.max(line_range.start);
        let end = range.end.min(line_range.end);

        // the range coordinates have to be localized to the line in question
        // before we report the highlights
        let local_line_range_start = start - line_range.start;
        let local_line_range_end = end - line_range.start;

        result.push((local_line_range_start..local_line_range_end, *style));

        // break loop if we are outside of the line range
        if line_range.end <= range.end {
            break;
        }

        highlights_iter.next();
    }
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
