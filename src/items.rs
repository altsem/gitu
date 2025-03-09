use crate::config::Config;
use crate::config::StyleConfig;
use crate::git::diff::Diff;
use crate::syntax_parser;
use crate::syntax_parser::SyntaxTag;
use crate::Res;
use core::str;
use git2::Commit;
use git2::Oid;
use git2::Repository;
use gitu_diff::Status;
use itertools::Itertools;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;
use regex::Regex;
use std::borrow::Cow;
use std::iter;
use std::ops::Range;
use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;

#[derive(Default, Clone, Debug)]
pub(crate) struct Item {
    pub(crate) id: Cow<'static, str>,
    pub(crate) display: Line<'static>,
    pub(crate) section: bool,
    pub(crate) default_collapsed: bool,
    pub(crate) depth: usize,
    pub(crate) unselectable: bool,
    pub(crate) target_data: Option<TargetData>,
}

#[derive(Clone, Debug)]
pub(crate) enum TargetData {
    AllStaged,
    AllUnstaged,
    AllUntracked(Vec<PathBuf>),
    Branch(String),
    Commit(String),
    File(PathBuf),
    Delta {
        diff: Rc<Diff>,
        file_i: usize,
    },
    Hunk {
        diff: Rc<Diff>,
        file_i: usize,
        hunk_i: usize,
    },
    HunkLine {
        diff: Rc<Diff>,
        file_i: usize,
        hunk_i: usize,
        line_i: usize,
    },
    Stash {
        commit: String,
        id: usize,
    },
}

pub(crate) fn create_diff_items(
    config: Rc<Config>,
    diff: &Rc<Diff>,
    depth: usize,
    default_collapsed: bool,
) -> impl Iterator<Item = Item> + '_ {
    diff.file_diffs
        .iter()
        .enumerate()
        .flat_map(move |(file_i, file_diff)| {
            let target_data = TargetData::Delta {
                diff: Rc::clone(diff),
                file_i,
            };
            let config = Rc::clone(&config);

            iter::once(Item {
                id: Rc::clone(diff).text[file_diff.header.range.clone()]
                    .to_string()
                    .into(),
                display: Line::styled(
                    format!(
                        "{:8}   {}",
                        format!("{:?}", file_diff.header.status).to_lowercase(),
                        match file_diff.header.status {
                            Status::Renamed => format!(
                                "{} -> {}",
                                &Rc::clone(diff).text[file_diff.header.old_file.clone()],
                                &Rc::clone(diff).text[file_diff.header.new_file.clone()]
                            ),
                            _ =>
                                Rc::clone(diff).text[file_diff.header.new_file.clone()].to_string(),
                        }
                    ),
                    &config.style.file_header,
                ),
                section: true,
                default_collapsed,
                depth,
                target_data: Some(target_data),
                ..Default::default()
            })
            .chain(file_diff.hunks.iter().cloned().enumerate().flat_map(
                move |(hunk_i, _hunk)| {
                    create_hunk_items(
                        Rc::clone(&config),
                        Rc::clone(diff),
                        file_i,
                        hunk_i,
                        depth + 1,
                    )
                },
            ))
        })
}

fn create_hunk_items(
    config: Rc<Config>,
    diff: Rc<Diff>,
    file_i: usize,
    hunk_i: usize,
    depth: usize,
) -> impl Iterator<Item = Item> {
    iter::once(Item {
        // TODO Don't do this
        id: diff.format_patch(file_i, hunk_i).into(),
        display: Line::styled(
            diff.text[diff.file_diffs[file_i].hunks[hunk_i].header.range.clone()].to_string(),
            &config.style.hunk_header,
        ),
        section: true,
        depth,
        target_data: Some(TargetData::Hunk {
            diff: Rc::clone(&diff),
            file_i,
            hunk_i,
        }),
        ..Default::default()
    })
    .chain(format_diff_hunk_items(
        &config,
        diff,
        file_i,
        hunk_i,
        depth + 1,
    ))
}

fn format_diff_hunk_items(
    config: &Config,
    diff: Rc<Diff>,
    file_i: usize,
    hunk_i: usize,
    depth: usize,
) -> Vec<Item> {
    // TODO include function context in syntax highlights?

    let old_path = &diff.text[diff.file_diffs[file_i].header.old_file.clone()];
    let new_path = &diff.text[diff.file_diffs[file_i].header.new_file.clone()];

    let hunk = &diff.file_diffs[file_i].hunks[hunk_i];
    let hunk_content = &diff.text[hunk.content.range.clone()];
    let old_mask = diff.mask_old_hunk(file_i, hunk_i);
    let new_mask = diff.mask_new_hunk(file_i, hunk_i);

    let old_tags_iter = &mut iter_highlights(&config.style, old_path, &old_mask);
    let new_tags_iter = &mut iter_highlights(&config.style, new_path, &new_mask);

    let diff_style_iter = &mut iter_diff_tags(&diff.text, hunk);

    iter_line_ranges(hunk_content)
        .enumerate()
        .map(|(line_i, (line_range, line))| {
            let spans = if line.starts_with('-') {
                build_diff_line(diff_style_iter, &line_range, hunk_content)
            } else if line.starts_with('+') {
                build_diff_line(diff_style_iter, &line_range, hunk_content)
            } else {
                build_diff_line(diff_style_iter, &line_range, hunk_content)
            };

            let unselectable = line.starts_with(' ');

            Item {
                display: Line::from(spans),
                unselectable,
                depth,
                target_data: Some(TargetData::HunkLine {
                    diff: Rc::clone(&diff),
                    file_i,
                    hunk_i,
                    line_i,
                }),
                ..Default::default()
            }
        })
        .collect()
}

fn iter_diff_tags<'a>(
    source: &'a str,
    hunk: &'a gitu_diff::Hunk,
) -> iter::Peekable<impl Iterator<Item = (Range<usize>, Style)> + 'a> {
    let hunk_content = source[hunk.content.range.clone()].as_bytes();

    hunk.content
        .changes
        .iter()
        .filter_map(|change| {
            let (Some(old_change), Some(new_change)) = (&change.old, &change.new) else {
                return None;
            };

            let old_start = old_change.start - hunk.content.range.start;
            let old_end = old_change.end - hunk.content.range.start;
            let new_start = new_change.start - hunk.content.range.start;
            let new_end = new_change.end - hunk.content.range.start;

            // TODO Might result in a lot of Vec allocations
            let (old, new): (Vec<_>, Vec<_>) = similar::capture_diff(
                similar::Algorithm::Lcs,
                hunk_content,
                old_start..old_end,
                hunk_content,
                new_start..new_end,
            )
            .into_iter()
            .map(|op| match op.tag() {
                similar::DiffTag::Equal => (None, None),
                similar::DiffTag::Delete => (Some((op.old_range(), Style::from(Color::Red))), None),
                similar::DiffTag::Insert => {
                    (Some((op.new_range(), Style::from(Color::Green))), None)
                }
                similar::DiffTag::Replace => (
                    Some((op.old_range(), Style::from(Color::Red))),
                    Some((op.new_range(), Style::from(Color::Green))),
                ),
            })
            .unzip();

            Some(old.into_iter().chain(new).flatten())
        })
        .flatten()
        .peekable()
}

fn iter_line_ranges(content: &str) -> impl Iterator<Item = (Range<usize>, &str)> + '_ {
    content
        .split_inclusive('\n')
        .scan(0..0, |prev_line_range, line| {
            let line_range = prev_line_range.end..(prev_line_range.end + line.len());
            *prev_line_range = line_range.clone();
            Some((line_range, line))
        })
}

fn iter_highlights<'a>(
    config: &'a StyleConfig,
    path: &'a str,
    content: &'a str,
) -> iter::Peekable<impl Iterator<Item = (Range<usize>, Style)> + 'a> {
    syntax_parser::highlight(Path::new(path), content)
        .into_iter()
        .map(move |(range, tag)| (range, syntax_highlight_tag_style(config, tag)))
        .peekable()
}

fn build_diff_line<'a>(
    syntax_tags_iter: &mut iter::Peekable<impl Iterator<Item = (Range<usize>, Style)>>,
    line_range: &Range<usize>,
    content: &str,
) -> Vec<Span<'a>> {
    syntax_tags_iter
        .peeking_take_while(|(syntax_range, _)| syntax_range.end <= line_range.start)
        .map(|(_, tag)| tag)
        .for_each(drop);

    // FIXME: There's something wrong about line boundaries. Can't use previous tag if it ends before the line start?

    iter::once((line_range.start, Style::new()))
        .chain(
            syntax_tags_iter
                .peeking_take_while(|(syntax_range, _)| syntax_range.start < line_range.end)
                .flat_map(|(range, style)| [(range.start, style), (range.end, Style::new())]),
        )
        .chain([(line_range.end, Style::new())])
        .tuple_windows()
        .map(|((start, style), (end, _))| (start..end, style))
        .map(|(range, style)| Span::styled(content[range].replace('\t', "    "), style))
        .collect::<Vec<_>>()
}

fn syntax_highlight_tag_style(config: &StyleConfig, tag: SyntaxTag) -> Style {
    match tag {
        SyntaxTag::Attribute => &config.syntax_highlight.attribute,
        SyntaxTag::Comment => &config.syntax_highlight.comment,
        SyntaxTag::Constant => &config.syntax_highlight.constant,
        SyntaxTag::ConstantBuiltin => &config.syntax_highlight.constant_builtin,
        SyntaxTag::Constructor => &config.syntax_highlight.constructor,
        SyntaxTag::Embedded => &config.syntax_highlight.embedded,
        SyntaxTag::Function => &config.syntax_highlight.function,
        SyntaxTag::FunctionBuiltin => &config.syntax_highlight.function_builtin,
        SyntaxTag::Keyword => &config.syntax_highlight.keyword,
        SyntaxTag::Module => &config.syntax_highlight.module,
        SyntaxTag::Number => &config.syntax_highlight.number,
        SyntaxTag::Operator => &config.syntax_highlight.operator,
        SyntaxTag::Property => &config.syntax_highlight.property,
        SyntaxTag::PunctuationBracket => &config.syntax_highlight.punctuation_bracket,
        SyntaxTag::PunctuationDelimiter => &config.syntax_highlight.punctuation_delimiter,
        SyntaxTag::String => &config.syntax_highlight.string,
        SyntaxTag::StringSpecial => &config.syntax_highlight.string_special,
        SyntaxTag::Tag => &config.syntax_highlight.tag,
        SyntaxTag::TypeBuiltin => &config.syntax_highlight.type_builtin,
        SyntaxTag::TypeRegular => &config.syntax_highlight.type_regular,
        SyntaxTag::VariableBuiltin => &config.syntax_highlight.variable_builtin,
        SyntaxTag::VariableParameter => &config.syntax_highlight.variable_parameter,
    }
    .into()
}

pub(crate) fn stash_list(config: &Config, repo: &Repository, limit: usize) -> Res<Vec<Item>> {
    let style = &config.style;

    Ok(repo
        .reflog("refs/stash")?
        .iter()
        .enumerate()
        .map(|(i, stash)| -> Res<Item> {
            let spans = itertools::intersperse(
                iter::once(Span::styled(format!("stash@{i}"), &style.hash)).chain([stash
                    .message()
                    .unwrap_or("")
                    .to_string()
                    .into()]),
                Span::raw(" "),
            )
            .collect::<Vec<_>>();

            Ok(Item {
                id: stash.id_new().to_string().into(),
                display: Line::from(spans),
                depth: 1,
                target_data: Some(TargetData::Stash {
                    commit: stash.id_new().to_string(),
                    id: i,
                }),
                ..Default::default()
            })
        })
        .map(|result| match result {
            Ok(item) => item,
            Err(err) => Item {
                id: err.to_string().into(),
                display: err.to_string().into(),
                ..Default::default()
            },
        })
        .take(limit)
        .collect::<Vec<_>>())
}

pub(crate) fn log(
    config: &Config,
    repo: &Repository,
    limit: usize,
    rev: Option<Oid>,
    msg_regex: Option<Regex>,
) -> Res<Vec<Item>> {
    let style = &config.style;
    let mut revwalk = repo.revwalk()?;
    if let Some(r) = rev {
        revwalk.push(r)?;
    } else if revwalk.push_head().is_err() {
        return Ok(vec![]);
    }

    let references = repo
        .references()?
        .filter_map(Result::ok)
        .filter_map(
            |reference| match (reference.peel_to_commit(), reference.shorthand()) {
                (Ok(target), Some(name)) => {
                    if name.ends_with("/HEAD") || name.starts_with("prefetch/remotes/") {
                        return None;
                    }

                    let style: Style = if reference.is_remote() {
                        &style.remote
                    } else if reference.is_tag() {
                        &style.tag
                    } else {
                        &style.branch
                    }
                    .into();

                    Some((target, Span::styled(name.to_string(), style)))
                }
                _ => None,
            },
        )
        .collect::<Vec<(Commit, Span)>>();

    let items: Vec<Item> = revwalk
        .map(|oid_result| -> Res<Option<Item>> {
            let oid = oid_result?;
            let commit = repo.find_commit(oid)?;
            let short_id = commit.as_object().short_id()?.as_str().unwrap().to_string();

            let spans = itertools::intersperse(
                iter::once(Span::styled(short_id, &style.hash))
                    .chain(
                        references
                            .iter()
                            .filter(|(commit, _)| commit.id() == oid)
                            .map(|(_, name)| name.clone()),
                    )
                    .chain([commit.summary().unwrap_or("").to_string().into()]),
                Span::raw(" "),
            )
            .collect::<Vec<_>>();

            if let Some(re) = &msg_regex {
                if !re.is_match(commit.message().unwrap_or("")) {
                    return Ok(None);
                }
            }

            Ok(Some(Item {
                id: oid.to_string().into(),
                display: Line::from(spans),
                depth: 1,
                target_data: Some(TargetData::Commit(oid.to_string())),
                ..Default::default()
            }))
        })
        .filter_map(|result| match result {
            Ok(item) => item,
            Err(err) => Some(Item {
                id: err.to_string().into(),
                display: err.to_string().into(),
                ..Default::default()
            }),
        })
        .take(limit)
        .collect();

    if items.is_empty() {
        Ok(vec![Item {
            display: Line::raw("No commits found"),
            ..Default::default()
        }])
    } else {
        Ok(items)
    }
}

pub(crate) fn blank_line() -> Item {
    Item {
        display: Line::raw(""),
        depth: 0,
        unselectable: true,
        ..Default::default()
    }
}
