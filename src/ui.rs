use std::borrow::Cow;
use std::ops::Range;

use crate::Res;
use crate::app::State;
use crate::error::Error;
use crate::screen;
use crate::search;
use crate::style::{Color, Modifier, Style};
use crate::term::TermBackend;
use crate::text_input::Status;
use crate::ui::layout::{LayoutItem, Measure, Payload};
use itertools::Itertools;
use layout::LayoutTree;
use layout::opts;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

mod cmd_log;
pub(crate) mod item;
pub(crate) mod layout;
mod menu;
pub mod picker;

const CARET: &str = "\u{2588}";
const DASHES: &str = "────────────────────────────────────────────────────────────────";
const BLANKS: &str = "                                                                ";

#[derive(Debug, Clone)]
pub(crate) struct Span<'a>(pub(crate) Cow<'a, str>, pub(crate) Style);
pub(crate) type UiTree<'a> = LayoutTree<Span<'a>, Style>;

impl Span<'_> {
    pub(crate) fn text(&self) -> &str {
        self.0.as_ref()
    }
}

impl Measure for Span<'_> {
    type Unit = u16;

    fn measure(&self) -> [u16; 2] {
        [UnicodeWidthStr::width(self.0.as_ref()) as u16, 1]
    }
}

pub(crate) fn ui(term: &mut TermBackend, state: &mut State) -> Res<()> {
    let size = term.size().unwrap();
    let mut layout = UiTree::new();

    let mut screen_leaves = 0..0;

    layout.col(opts(), |layout| {
        layout.col(opts().fill_xy(), |layout| {
            let hide_cursor = state.picker.is_some();
            let start = layout.node_count();
            screen::layout_screen(layout, state.screens.last().unwrap(), hide_cursor);
            screen_leaves = start..layout.node_count();
        });

        layout.col(opts(), |layout| {
            menu::layout_menu(layout, state, size.0 as usize);
            cmd_log::layout_cmd_log(
                layout,
                &state.current_cmd_log,
                &state.config,
                size.0 as usize,
            );
            layout_prompt(layout, state, size.0 as usize);
            layout_picker(layout, state, size.0 as usize);
            if !state.pending_keys.is_empty() {
                let keys = &state
                    .pending_keys
                    .iter()
                    .map(|(_, k)| k.to_string())
                    .collect::<String>();

                layout_span(layout, (("    ".to_string() + keys).into(), Style::new()));
            }
        });
    });

    let highlight = Highlight {
        matches: search_matches(&layout, state, screen_leaves),
        style: Style::from(&state.config.style.search_match),
    };

    let computed = layout.compute([size.0, size.1]);

    let mut items = computed.iter().collect::<Vec<_>>();
    items.sort_by_key(|item| [item.pos[1], item.pos[0]]);

    print_spans(term, size, items, &highlight)?;

    term.flush().map_err(Error::Term)?;
    state.screens.last_mut().unwrap().size = size;

    Ok(())
}

fn search_matches(
    layout: &UiTree,
    state: &State,
    screen_leaves: Range<usize>,
) -> Vec<(usize, Range<usize>)> {
    let Some(matcher) = state.screens.last().unwrap().get_search_matcher() else {
        return vec![];
    };

    let mut run_text = search::RunText::default();
    let mut matches = vec![];

    for run in layout.leaf_runs() {
        let mut run = run.peekable();

        if !run
            .peek()
            .is_some_and(|&(leaf, _)| screen_leaves.contains(&leaf))
        {
            continue;
        }

        let text = run_text.read(run.map(|(leaf, span)| (leaf, span.text())));

        for matched in matcher
            .find_iter(text)
            .map(|m| m.range())
            .collect::<Vec<_>>()
        {
            matches.extend(run_text.per_leaf(matched));
        }
    }

    matches
}

struct Highlight {
    /// Sorted by leaf, and within a leaf in ascending order.
    matches: Vec<(usize, Range<usize>)>,
    style: Style,
}

impl Highlight {
    /// What matched within `leaf`, as ranges of that leaf's own text.
    fn within(&self, leaf: usize) -> impl Iterator<Item = Range<usize>> {
        let from = self.matches.partition_point(|&(at, _)| at < leaf);

        self.matches[from..]
            .iter()
            .take_while(move |&&(at, _)| at == leaf)
            .map(|(_, matched)| matched.clone())
    }
}

fn layout_prompt<'a>(layout: &mut UiTree<'a>, state: &'a State, width: usize) {
    let Some(ref prompt_data) = state.prompt.data else {
        return;
    };

    let (symbol, symbol_color) = match state.prompt.state.status {
        Status::Pending => ("?", Color::Cyan),
        Status::Aborted => ("✘", Color::Red),
        Status::Done => ("✔", Color::Green),
    };

    let separator_style = Style::from(&state.config.style.separator);
    let prompt_style = Style::from(&state.config.style.prompt);

    repeat_chars(layout, width, DASHES, separator_style);
    layout.row(opts(), |layout| {
        layout_span(
            layout,
            (
                symbol.into(),
                Style {
                    fg: Some(symbol_color),
                    ..Style::new()
                },
            ),
        );
        layout_span(layout, (" ".into(), Style::new()));
        layout_span(
            layout,
            (prompt_data.prompt_text.as_ref().into(), prompt_style),
        );
        layout_span(layout, (" › ".into(), prompt_style));
        let (before, at_cursor, after) = state.prompt.state.split_at_cursor();
        layout_span(layout, (before.into(), Style::new()));
        layout_cursor(layout, at_cursor);
        layout_span(layout, (after.into(), Style::new()));
    });
}

/// Draws the cursor over the char it's on, so that it takes up no width of its
/// own. Past the end of the value there's nothing to draw it over.
pub(crate) fn layout_cursor<'a>(layout: &mut UiTree<'a>, at_cursor: &'a str) {
    if at_cursor.is_empty() {
        layout_span(layout, (CARET.into(), Style::new()));
    } else {
        layout_span(
            layout,
            (
                at_cursor.into(),
                Style {
                    add_modifier: Modifier::REVERSED,
                    ..Style::new()
                },
            ),
        );
    }
}

fn layout_picker<'a>(layout: &mut UiTree<'a>, state: &'a State, width: usize) {
    if let Some(ref picker_state) = state.picker {
        picker::layout_picker(layout, picker_state, &state.config, width);
    }
}

/// Lays out `content` as a single row of its own.
pub(crate) fn layout_line<'a>(layout: &mut UiTree<'a>, content: Cow<'a, str>, style: Style) {
    layout.row(opts(), |layout| {
        layout_span(layout, (content, style));
    });
}

pub(crate) fn layout_span<'a>(layout: &mut UiTree<'a>, span: (Cow<'a, str>, Style)) {
    match span.0 {
        Cow::Borrowed(s) => {
            for word in words(s) {
                layout.leaf(Span(Cow::Borrowed(word), span.1));
            }
        }
        Cow::Owned(s) => {
            for word in words(&s) {
                layout.leaf(Span(Cow::Owned(word.into()), span.1));
            }
        }
    }
}

/// Splits into the words to wrap on, dropping line breaks. A span occupies a
/// single row, so a line break would otherwise be printed as-is and shift the
/// rest of the frame.
fn words(text: &str) -> impl Iterator<Item = &str> {
    text.split_word_bounds()
        .filter(|word| !word.bytes().all(|byte| byte == b'\n' || byte == b'\r'))
}

pub(crate) fn repeat_chars(layout: &mut UiTree, count: usize, chars: &'static str, style: Style) {
    let grapheme_count = chars.grapheme_indices(true).count();
    let full = count / grapheme_count;
    let partial = count % grapheme_count;

    layout.row(opts(), |layout| {
        for _ in 0..full {
            layout_span(layout, (chars.into(), style));
        }

        if partial > 0 {
            let end = chars
                .grapheme_indices(true)
                .tuple_windows()
                .take(partial)
                .last()
                .map(|((_, _), (end, _))| end)
                .unwrap_or(chars.len());

            layout_span(layout, (chars[..end].into(), style));
        }
    });
}

fn print_spans(
    term: &mut TermBackend,
    size: (u16, u16),
    items: Vec<LayoutItem<Payload<'_, Span<'_>, Style>, u16>>,
    highlight: &Highlight,
) -> Result<(), Error> {
    let mut at = [0, 0];
    let mut bg = Style::new();
    let mut bg_end = 0;
    for item in items {
        let LayoutItem {
            index,
            data,
            pos,
            size: item_size,
        } = item;

        blank_until(term, &mut at, [0, pos[1]], size.0, bg, bg_end)?;

        match data {
            Payload::Leaf(span) => {
                blank_until(term, &mut at, pos, size.0, bg, bg_end)?;
                term.queue_move_cursor(pos[0], pos[1])?;
                print_span(term, span, highlight.within(index), highlight.style)?;

                at[0] = pos[0].saturating_add(item_size[0]);
            }
            Payload::Container(style) => {
                bg = *style;
                bg_end = pos[1].saturating_add(item_size[1]);
            }
        }
    }
    blank_until(term, &mut at, [0, size.1], size.0, bg, bg_end)?;
    Ok(())
}

fn print_span(
    term: &mut TermBackend,
    Span(text, style): &Span,
    matches: impl Iterator<Item = Range<usize>>,
    match_style: Style,
) -> Result<(), Error> {
    let mut at = 0;

    for matched in matches {
        if at < matched.start {
            term.queue_print(&text[at..matched.start], style)?;
        }

        at = matched.end;
        term.queue_print(&text[matched], &style.patch(match_style))?;
    }

    if at < text.len() {
        term.queue_print(&text[at..], style)?;
    }

    Ok(())
}

fn blank_until(
    term: &mut TermBackend,
    at: &mut [u16; 2],
    to: [u16; 2],
    width: u16,
    bg: Style,
    bg_end: u16,
) -> Res<()> {
    let row_bg = |y| if y < bg_end { bg } else { Style::new() };

    while at[1] < to[1] {
        queue_blanks(term, *at, width.saturating_sub(at[0]), &row_bg(at[1]))?;
        *at = [0, at[1] + 1];
    }

    queue_blanks(term, *at, to[0].saturating_sub(at[0]), &row_bg(at[1]))?;
    at[0] = at[0].max(to[0]);

    Ok(())
}

fn queue_blanks(term: &mut TermBackend, at: [u16; 2], width: u16, style: &Style) -> Res<()> {
    if width == 0 {
        return Ok(());
    }

    term.queue_move_cursor(at[0], at[1])?;

    let mut left = width as usize;
    while left > 0 {
        let blanks = left.min(BLANKS.len());
        term.queue_print(&BLANKS[..blanks], style)?;
        left -= blanks;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::screen::matcher;

    /// Lays `spans` out as one run and reads back what a search goes against.
    fn run_of(spans: &[&'static str]) -> (UiTree<'static>, search::RunText) {
        let mut layout = UiTree::new();

        layout.row(opts(), |layout| {
            for span in spans {
                layout_span(layout, ((*span).into(), Style::new()));
            }
        });

        (layout, search::RunText::default())
    }

    /// What `query` matched, as the text of each match.
    fn matched(spans: &[&'static str], query: &str) -> Vec<String> {
        let (layout, mut run_text) = run_of(spans);
        let matcher = matcher(query).unwrap();

        layout
            .leaf_runs()
            .flat_map(|run| {
                let text = run_text.read(run.map(|(leaf, span)| (leaf, span.text())));
                matcher
                    .find_iter(text)
                    .map(|matched| matched.as_str().to_string())
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    #[test]
    fn a_match_may_run_from_one_span_into_the_next() {
        assert_eq!(
            vec!["main add"],
            matched(&["main", " ", "add thirdfile"], "main add")
        );
    }

    #[test]
    fn matching_ignores_case_without_shifting_what_it_hands_back() {
        assert_eq!(
            vec!["Add Thirdfile"],
            matched(&["Add Thirdfile"], "add thirdfile")
        );
    }

    #[test]
    fn multi_byte_characters_are_matched_whole() {
        assert_eq!(vec!["⊕"], matched(&["ändra ⊕ hunk"], "⊕"));
        assert_eq!(vec!["ändra"], matched(&["ändra ⊕ hunk"], "ändra"));
    }

    /// Runs are what search goes against, and a nested container is a break in
    /// one, so text on either side of it is never one match.
    #[test]
    fn a_match_may_not_run_across_a_nested_container() {
        let mut layout = UiTree::new();

        layout.row(opts(), |layout| {
            layout_span(layout, ("1e81efc".into(), Style::new()));

            // What pushes the author and age to the right on a commit row.
            layout.row(opts().fill_x(), |layout| {
                layout_span(layout, (" main".into(), Style::new()));
            });
        });

        let mut run_text = search::RunText::default();
        let matcher = matcher("1e81efc main").unwrap();
        let matched = layout.leaf_runs().any(|run| {
            matcher.is_match(run_text.read(run.map(|(leaf, span)| (leaf, span.text()))))
        });

        assert!(!matched);
    }

    /// A match is painted by each leaf it covers, so it has to be cut up along
    /// the ones it spans.
    #[test]
    fn a_match_is_handed_back_per_leaf_it_covers() {
        let (layout, mut run_text) = run_of(&["main", " ", "add thirdfile"]);
        let run = layout.leaf_runs().next().unwrap();
        let text = run_text
            .read(run.map(|(leaf, span)| (leaf, span.text())))
            .to_string();

        let matched = matcher("main add").unwrap().find(&text).unwrap().range();
        let per_leaf = run_text.per_leaf(matched).collect::<Vec<_>>();

        // "main" whole, the space whole, then "add" of "add thirdfile". The
        // leaves are numbered from 2, after the root and the row holding them.
        assert_eq!(vec![(2, 0..4), (3, 0..1), (4, 0..3)], per_leaf);
    }
}
