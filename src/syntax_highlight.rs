use crate::config::Config;

use itertools::Itertools;
use ratatui::style::Style;
use std::{cell::RefCell, collections::HashMap, iter, ops::Range, path::Path};
use tree_sitter::Language;
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

fn styles(style: &crate::config::StyleConfig) -> [Style; 22] {
    [
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
    ]
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

/// The defaults for these seem to exist in the `package.json` of each repo:
/// `curl https://raw.githubusercontent.com/tree-sitter/tree-sitter-html/master/package.json | jq -r '."tree-sitter"'`
fn determine_lang(path: &Path) -> Option<Language> {
    let extension = path.extension().and_then(|s| s.to_str())?;

    match extension {
        "rs" => Some(tree_sitter_rust::language()),
        "toml" => Some(tree_sitter_toml::language()),
        "js" => Some(tree_sitter_javascript::language()),
        "c" | "h" => Some(tree_sitter_c::language()),
        "json" => Some(tree_sitter_json::language()),
        "cc" => Some(tree_sitter_cpp::language()),
        "rb" => Some(tree_sitter_ruby::language()),
        "hs" => Some(tree_sitter_haskell::language()),
        "go" => Some(tree_sitter_go::language()),
        "cs" => Some(tree_sitter_c_sharp::language()),
        "py" => Some(tree_sitter_python::language()),
        "ts" => Some(tree_sitter_typescript::language_typescript()),
        "tsx" => Some(tree_sitter_typescript::language_tsx()),
        "sh" | "bash" | ".bashrc" | ".bash_profile" | "ebuild" | "eclass" => {
            Some(tree_sitter_bash::language())
        }
        "php" => Some(tree_sitter_php::language()),
        "java" => Some(tree_sitter_java::language()),
        "scala" | "sbt" => Some(tree_sitter_scala::language()),
        "ml" => Some(tree_sitter_ocaml::language_ocaml()),
        "mli" => Some(tree_sitter_ocaml::language_ocaml_interface()),
        "html" => Some(tree_sitter_html::language()),
        "ex" | "exs" => Some(tree_sitter_elixir::language()),
        _ => None,
    }
}

fn create_highlight_config(lang: &Language) -> HighlightConfiguration {
    let (highlights_query, injections_query, locals_query) =
        if lang == &tree_sitter_rust::language() {
            (
                tree_sitter_rust::HIGHLIGHT_QUERY,
                tree_sitter_rust::INJECTIONS_QUERY,
                "",
            )
        } else if lang == &tree_sitter_toml::language() {
            (tree_sitter_toml::HIGHLIGHT_QUERY, "", "")
        } else if lang == &tree_sitter_javascript::language() {
            (
                tree_sitter_javascript::HIGHLIGHT_QUERY,
                tree_sitter_javascript::INJECTION_QUERY,
                tree_sitter_javascript::LOCALS_QUERY,
            )
        } else if lang == &tree_sitter_c::language() {
            (tree_sitter_c::HIGHLIGHT_QUERY, "", "")
        } else if lang == &tree_sitter_json::language() {
            (tree_sitter_json::HIGHLIGHT_QUERY, "", "")
        } else if lang == &tree_sitter_cpp::language() {
            (tree_sitter_cpp::HIGHLIGHT_QUERY, "", "")
        } else if lang == &tree_sitter_ruby::language() {
            (
                tree_sitter_ruby::HIGHLIGHT_QUERY,
                "",
                tree_sitter_ruby::LOCALS_QUERY,
            )
        } else if lang == &tree_sitter_haskell::language() {
            (
                tree_sitter_haskell::HIGHLIGHTS_QUERY,
                "",
                tree_sitter_haskell::LOCALS_QUERY,
            )
        } else if lang == &tree_sitter_go::language() {
            (tree_sitter_go::HIGHLIGHT_QUERY, "", "")
        } else if lang == &tree_sitter_c_sharp::language() {
            (tree_sitter_c_sharp::HIGHLIGHT_QUERY, "", "")
        } else if lang == &tree_sitter_python::language() {
            (tree_sitter_python::HIGHLIGHT_QUERY, "", "")
        } else if lang == &tree_sitter_typescript::language_typescript()
            || lang == &tree_sitter_typescript::language_tsx()
        {
            (
                tree_sitter_typescript::HIGHLIGHT_QUERY,
                "",
                tree_sitter_typescript::LOCALS_QUERY,
            )
        } else if lang == &tree_sitter_bash::language() {
            (tree_sitter_bash::HIGHLIGHT_QUERY, "", "")
        } else if lang == &tree_sitter_php::language() {
            (
                tree_sitter_php::HIGHLIGHT_QUERY,
                tree_sitter_php::INJECTIONS_QUERY,
                "",
            )
        } else if lang == &tree_sitter_java::language() {
            (tree_sitter_java::HIGHLIGHT_QUERY, "", "")
        } else if lang == &tree_sitter_scala::language() {
            (
                tree_sitter_scala::HIGHLIGHTS_QUERY,
                "",
                tree_sitter_scala::LOCALS_QUERY,
            )
        } else if lang == &tree_sitter_ocaml::language_ocaml() {
            (
                tree_sitter_ocaml::HIGHLIGHTS_QUERY,
                "",
                tree_sitter_ocaml::LOCALS_QUERY,
            )
        } else if lang == &tree_sitter_html::language() {
            (
                tree_sitter_html::HIGHLIGHTS_QUERY,
                tree_sitter_html::INJECTIONS_QUERY,
                "",
            )
        } else if lang == &tree_sitter_elixir::language() {
            (tree_sitter_elixir::HIGHLIGHTS_QUERY, "", "")
        } else {
            panic!("Undefined language");
        };

    let mut highlight_config =
        HighlightConfiguration::new(*lang, highlights_query, injections_query, locals_query)
            .unwrap();

    highlight_config.configure(HIGHLIGHT_NAMES);
    highlight_config
}

thread_local! {
    pub static HIGHLIGHTER: RefCell<Highlighter> = RefCell::new(Highlighter::new());
    pub static LANG_CONFIGS: RefCell<HashMap<Language, HighlightConfiguration>> = RefCell::new(HashMap::new());
}

pub(crate) fn highlight<'a>(
    config: &'a Config,
    path: &'a Path,
    content: &'a str,
) -> Vec<(Range<usize>, Style)> {
    let style = &config.style;
    let styles = styles(style);

    let Some(lang) = determine_lang(path) else {
        return vec![];
    };

    LANG_CONFIGS.with(|highlight_configs| {
        let mut highlight_configs_borrow = highlight_configs.borrow_mut();
        let config = highlight_configs_borrow
            .entry(lang)
            .or_insert_with_key(create_highlight_config);

        HIGHLIGHTER.with_borrow_mut(|highlighter| {
            highlighter
                .highlight(config, content.as_bytes(), None, |_| None)
                .unwrap()
                .scan((0..0, Style::new()), move |current, event| {
                    match event.unwrap() {
                        HighlightEvent::Source { start, end } => {
                            Some(Some((start..end, current.1)))
                        }
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
                .collect()
        })
    })
}
