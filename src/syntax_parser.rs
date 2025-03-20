use std::{cell::RefCell, collections::HashMap, ops::Range, path::Path};
use tree_sitter::Language;
use tree_sitter_highlight::{Highlight, HighlightConfiguration, HighlightEvent, Highlighter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxTag {
    Attribute,
    Comment,
    Constant,
    ConstantBuiltin,
    Constructor,
    Embedded,
    Function,
    FunctionBuiltin,
    Keyword,
    Module,
    Number,
    Operator,
    Property,
    PunctuationBracket,
    PunctuationDelimiter,
    String,
    StringSpecial,
    Tag,
    TypeBuiltin,
    TypeRegular,
    VariableBuiltin,
    VariableParameter,
}

impl AsRef<str> for SyntaxTag {
    fn as_ref(&self) -> &str {
        match self {
            SyntaxTag::Attribute => "attribute",
            SyntaxTag::Comment => "comment",
            SyntaxTag::ConstantBuiltin => "constant.builtin",
            SyntaxTag::Constant => "constant",
            SyntaxTag::Constructor => "constructor",
            SyntaxTag::Embedded => "embedded",
            SyntaxTag::FunctionBuiltin => "function.builtin",
            SyntaxTag::Function => "function",
            SyntaxTag::Keyword => "keyword",
            SyntaxTag::Number => "number",
            SyntaxTag::Module => "module",
            SyntaxTag::Property => "property",
            SyntaxTag::Operator => "operator",
            SyntaxTag::PunctuationBracket => "punctuation.bracket",
            SyntaxTag::PunctuationDelimiter => "punctuation.delimiter",
            SyntaxTag::StringSpecial => "string.special",
            SyntaxTag::String => "string",
            SyntaxTag::Tag => "tag",
            SyntaxTag::TypeRegular => "type",
            SyntaxTag::TypeBuiltin => "type.builtin",
            SyntaxTag::VariableBuiltin => "variable.builtin",
            SyntaxTag::VariableParameter => "variable.parameter",
        }
    }
}

fn tags_by_highlight_index() -> [SyntaxTag; 22] {
    [
        SyntaxTag::Attribute,
        SyntaxTag::Comment,
        SyntaxTag::ConstantBuiltin,
        SyntaxTag::Constant,
        SyntaxTag::Constructor,
        SyntaxTag::Embedded,
        SyntaxTag::FunctionBuiltin,
        SyntaxTag::Function,
        SyntaxTag::Keyword,
        SyntaxTag::Number,
        SyntaxTag::Module,
        SyntaxTag::Property,
        SyntaxTag::Operator,
        SyntaxTag::PunctuationBracket,
        SyntaxTag::PunctuationDelimiter,
        SyntaxTag::StringSpecial,
        SyntaxTag::String,
        SyntaxTag::Tag,
        SyntaxTag::TypeRegular,
        SyntaxTag::TypeBuiltin,
        SyntaxTag::VariableBuiltin,
        SyntaxTag::VariableParameter,
    ]
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

    highlight_config.configure(&tags_by_highlight_index());
    highlight_config
}

thread_local! {
    pub static HIGHLIGHTER: RefCell<Highlighter> = RefCell::new(Highlighter::new());
    pub static LANG_CONFIGS: RefCell<HashMap<Language, HighlightConfiguration>> = RefCell::new(HashMap::new());
}

pub(crate) fn parse<'a>(path: &'a Path, content: &'a str) -> Vec<(Range<usize>, SyntaxTag)> {
    let tags = tags_by_highlight_index();

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
                .scan(None, move |current_tag, event| match event.unwrap() {
                    HighlightEvent::Source { start, end } => Some(Some((start..end, *current_tag))),
                    HighlightEvent::HighlightStart(Highlight(highlight)) => {
                        *current_tag = Some(tags[highlight]);
                        Some(None)
                    }
                    HighlightEvent::HighlightEnd => {
                        *current_tag = None;
                        Some(None)
                    }
                })
                .flatten()
                .filter_map(|(range, maybe_tag)| maybe_tag.map(|tag| (range, tag)))
                .collect()
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlight() {
        let path = Path::new("test.rs");
        let content = r#"
fn main() {
    println!("Hello, world!");
}
"#;

        let syntax = parse(path, content);
        let syntax_with_content = syntax
            .into_iter()
            .map(|(range, style)| {
                (
                    std::str::from_utf8(&content.as_bytes()[range.clone()]).unwrap(),
                    style,
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(
            syntax_with_content,
            vec![
                ("fn", SyntaxTag::Keyword),
                ("main", SyntaxTag::Function),
                ("(", SyntaxTag::PunctuationBracket),
                (")", SyntaxTag::PunctuationBracket),
                ("{", SyntaxTag::PunctuationBracket),
                ("println", SyntaxTag::Function),
                ("!", SyntaxTag::Function),
                ("(", SyntaxTag::PunctuationBracket),
                ("\"Hello, world!\"", SyntaxTag::String),
                (")", SyntaxTag::PunctuationBracket),
                (";", SyntaxTag::PunctuationDelimiter),
                ("}", SyntaxTag::PunctuationBracket),
            ]
        );
    }
}
