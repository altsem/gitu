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

#[derive(PartialEq, Eq, Hash, Debug)]
enum Lang {
    Rust,
    Toml,
    Javascript,
    C,
    Json,
    Cpp,
    Ruby,
    Haskell,
    Go,
    CSharp,
    Python,
    Typescript,
    Tsx,
    Bash,
    Php,
    Java,
    Scala,
    Ocaml,
    OcamlInterface,
    Html,
    Elixir,
}

/// The defaults for these seem to exist in the `package.json` of each repo:
/// `curl https://raw.githubusercontent.com/tree-sitter/tree-sitter-html/master/package.json | jq -r '."tree-sitter"'`
fn determine_lang(path: &Path) -> Option<Lang> {
    let extension = path.extension().and_then(|s| s.to_str())?;

    match extension {
        "rs" => Some(Lang::Rust),
        "toml" => Some(Lang::Toml),
        "js" => Some(Lang::Javascript),
        "c" | "h" => Some(Lang::C),
        "json" => Some(Lang::Json),
        "cc" => Some(Lang::Cpp),
        "rb" => Some(Lang::Ruby),
        "hs" => Some(Lang::Haskell),
        "go" => Some(Lang::Go),
        "cs" => Some(Lang::CSharp),
        "py" => Some(Lang::Python),
        "ts" => Some(Lang::Typescript),
        "tsx" => Some(Lang::Tsx),
        "sh" | "bash" | ".bashrc" | ".bash_profile" | "ebuild" | "eclass" => Some(Lang::Bash),
        "php" => Some(Lang::Php),
        "java" => Some(Lang::Java),
        "scala" | "sbt" => Some(Lang::Scala),
        "ml" => Some(Lang::Ocaml),
        "mli" => Some(Lang::OcamlInterface),
        "html" => Some(Lang::Html),
        "ex" | "exs" => Some(Lang::Elixir),
        _ => None,
    }
}

fn create_highlight_config(lang: &Lang) -> HighlightConfiguration {
    let (lang_fn, hquery, iquery, lquery) = match lang {
        Lang::Rust => (
            tree_sitter_rust::LANGUAGE,
            tree_sitter_rust::HIGHLIGHTS_QUERY,
            tree_sitter_rust::INJECTIONS_QUERY,
            "",
        ),
        Lang::Toml => (
            tree_sitter_toml_ng::LANGUAGE,
            tree_sitter_toml_ng::HIGHLIGHTS_QUERY,
            "",
            "",
        ),
        Lang::Javascript => (
            tree_sitter_javascript::LANGUAGE,
            tree_sitter_javascript::HIGHLIGHT_QUERY,
            tree_sitter_javascript::INJECTIONS_QUERY,
            tree_sitter_javascript::LOCALS_QUERY,
        ),
        Lang::C => (
            tree_sitter_c::LANGUAGE,
            tree_sitter_c::HIGHLIGHT_QUERY,
            "",
            "",
        ),
        Lang::Json => (
            tree_sitter_json::LANGUAGE,
            tree_sitter_json::HIGHLIGHTS_QUERY,
            "",
            "",
        ),
        Lang::Cpp => (
            tree_sitter_cpp::LANGUAGE,
            tree_sitter_cpp::HIGHLIGHT_QUERY,
            "",
            "",
        ),
        Lang::Ruby => (
            tree_sitter_ruby::LANGUAGE,
            tree_sitter_ruby::HIGHLIGHTS_QUERY,
            "",
            tree_sitter_ruby::LOCALS_QUERY,
        ),
        Lang::Haskell => (
            tree_sitter_haskell::LANGUAGE,
            tree_sitter_haskell::HIGHLIGHTS_QUERY,
            "",
            tree_sitter_haskell::LOCALS_QUERY,
        ),
        Lang::Go => (
            tree_sitter_go::LANGUAGE,
            tree_sitter_go::HIGHLIGHTS_QUERY,
            "",
            "",
        ),
        Lang::CSharp => (tree_sitter_c_sharp::LANGUAGE, "", "", ""),
        Lang::Python => (
            tree_sitter_python::LANGUAGE,
            tree_sitter_python::HIGHLIGHTS_QUERY,
            "",
            "",
        ),
        Lang::Typescript => (
            tree_sitter_typescript::LANGUAGE_TYPESCRIPT,
            tree_sitter_typescript::HIGHLIGHTS_QUERY,
            "",
            tree_sitter_typescript::LOCALS_QUERY,
        ),
        Lang::Tsx => (
            tree_sitter_typescript::LANGUAGE_TSX,
            tree_sitter_typescript::HIGHLIGHTS_QUERY,
            "",
            tree_sitter_typescript::LOCALS_QUERY,
        ),
        Lang::Bash => (
            tree_sitter_bash::LANGUAGE,
            tree_sitter_bash::HIGHLIGHT_QUERY,
            "",
            "",
        ),
        Lang::Php => (
            tree_sitter_php::LANGUAGE_PHP,
            tree_sitter_php::HIGHLIGHTS_QUERY,
            tree_sitter_php::INJECTIONS_QUERY,
            "",
        ),
        Lang::Java => (
            tree_sitter_java::LANGUAGE,
            tree_sitter_java::HIGHLIGHTS_QUERY,
            "",
            "",
        ),
        Lang::Scala => (
            tree_sitter_scala::LANGUAGE,
            tree_sitter_scala::HIGHLIGHTS_QUERY,
            "",
            tree_sitter_scala::LOCALS_QUERY,
        ),
        Lang::Ocaml => (
            tree_sitter_ocaml::LANGUAGE_OCAML,
            tree_sitter_ocaml::HIGHLIGHTS_QUERY,
            "",
            tree_sitter_ocaml::LOCALS_QUERY,
        ),
        Lang::OcamlInterface => (
            tree_sitter_ocaml::LANGUAGE_OCAML_INTERFACE,
            tree_sitter_ocaml::HIGHLIGHTS_QUERY,
            "",
            tree_sitter_ocaml::LOCALS_QUERY,
        ),
        Lang::Html => (
            tree_sitter_html::LANGUAGE,
            tree_sitter_html::HIGHLIGHTS_QUERY,
            tree_sitter_html::INJECTIONS_QUERY,
            "",
        ),
        Lang::Elixir => (
            tree_sitter_elixir::LANGUAGE,
            tree_sitter_elixir::HIGHLIGHTS_QUERY,
            "",
            "",
        ),
    };

    let mut highlight_config = HighlightConfiguration::new(
        Language::new(lang_fn),
        format!("{lang:?}"),
        hquery,
        iquery,
        lquery,
    )
    .unwrap();

    highlight_config.configure(&tags_by_highlight_index());
    highlight_config
}

thread_local! {
    pub static HIGHLIGHTER: RefCell<Highlighter> = RefCell::new(Highlighter::new());
    pub static LANG_CONFIGS: RefCell<HashMap<Lang, HighlightConfiguration>> = RefCell::new(HashMap::new());
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
