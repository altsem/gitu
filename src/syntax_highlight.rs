use crate::config::Config;

use itertools::Itertools;
use ratatui::style::Style;
use std::{iter, ops::Range};
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

pub(crate) fn split_at_newlines<'a, D: Copy + 'a>(
    content: &'a str,
    (range, style): (Range<usize>, D),
) -> impl Iterator<Item = (Range<usize>, D)> + '_ {
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
