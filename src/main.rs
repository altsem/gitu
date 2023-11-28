use std::{
    collections::BTreeMap,
    io::{self, stdout},
    iter,
    path::Path,
};

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use git2::{Delta, Diff, DiffDelta, DiffHunk, DiffLine, Oid, Repository};
use ratatui::{
    prelude::CrosstermBackend,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::Paragraph,
    Frame, Terminal,
};

// TODO Keep collapsed state in set, fixes reloading repo state

#[derive(Debug)]
struct State {
    quit: bool,
    selected: usize,
    items: Vec<Item>,
}

#[derive(Default, Clone, Debug)]
struct Item {
    depth: usize,
    file: Option<String>,
    oid: Option<Oid>,
    header: Option<String>,
    status: Option<String>,
    diff_hunk: Option<String>,
    diff_line: Option<String>,
    section: Option<bool>,
}

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    let mut repo = Repository::open(".").unwrap();
    let items = create_status_items(&repo);

    let mut state = State {
        quit: false,
        selected: 0,
        items,
    };

    while !state.quit {
        terminal.draw(|frame| ui(frame, &state))?;
        handle_events(&mut state, &mut repo)?;
        state.selected = state.selected.clamp(0, state.items.len().saturating_sub(1));
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn create_status_items(repo: &Repository) -> Vec<Item> {
    let mut items = vec![];

    // items.extend(create_status_section(&repo, None, "Untracked files"));

    items.extend(create_status_section(
        &repo.diff_index_to_workdir(None, None).unwrap(),
        "Unstaged changes",
    ));

    items.extend(create_status_section(
        &repo
            .diff_tree_to_index(
                Some(&repo.head().unwrap().peel_to_tree().unwrap()),
                None,
                None,
            )
            .unwrap(),
        "Staged changes",
    ));

    let mut revwalk = repo.revwalk().unwrap();
    revwalk.push_head().unwrap();

    let recent_commits = revwalk
        .take(5)
        .map(|x| Item {
            oid: Some(x.unwrap()),
            depth: 1,
            ..Default::default()
        })
        .collect::<Vec<_>>();
    if !items.is_empty() {
        items.push(Item {
            header: Some("Recent commits".to_string()),
            depth: 0,
            section: Some(false),
            ..Default::default()
        });
        items.extend(recent_commits);
    }

    items
}

fn create_status_section<'a>(diff: &git2::Diff, header: &str) -> Vec<Item> {
    let deltas = diff.deltas();
    let count = deltas.len();

    if count == 0 {
        return vec![];
    }

    let mut items = vec![];

    diff.foreach(
        &mut |delta, i| true,
        None,
        None,
        Some(&mut |delta, hunk, line| {
            items.push(Item {
                depth: 3,
                file: delta
                    // TODO May need to look at old_file too
                    .new_file()
                    .path()
                    .map(|value| value.to_str().unwrap().to_string()),
                diff_hunk: Some(String::from_utf8(hunk.unwrap().header().to_owned()).unwrap()),
                diff_line: Some(
                    line.origin().to_string()
                        + &String::from_utf8(line.content().to_owned()).unwrap(),
                ),
                ..Default::default()
            });
            true
        }),
    )
    .unwrap();

    iter::once(Item {
        depth: 0,
        header: Some(header.to_string()),
        section: Some(false),
        ..Default::default()
    })
    .chain(
        file_hunk_groups(items)
            .into_iter()
            .flat_map(|(file, hunks)| {
                let section = Item {
                    file: Some(file),
                    section: Some(false),
                    depth: 1,
                    ..Default::default()
                };
                iter::once(section).chain(hunks.into_iter().flat_map(|(hunk, lines)| {
                    iter::once(Item {
                        section: Some(false),
                        diff_hunk: Some(hunk),
                        depth: 2,
                        ..Default::default()
                    })
                    .chain(lines.into_iter())
                }))
            }),
    )
    .collect()
}

fn format_diff(diff: &git2::Diff<'_>) -> String {
    let mut output = String::new();
    diff.print(
        git2::DiffFormat::Patch,
        |_delta: DiffDelta, _hunk: Option<DiffHunk>, line: DiffLine| -> bool {
            output.push_str(&format!(
                "{}{}",
                match line.origin() {
                    '+' | '-' | ' ' => line.origin().to_string(),
                    _ => "".to_string(),
                },
                std::str::from_utf8(line.content()).unwrap()
            ));
            true
        },
    )
    .unwrap();

    output
}

fn format_delta_status(delta: &Delta) -> &'_ str {
    match delta {
        Delta::Unmodified => "unmodified",
        Delta::Added => "added     ",
        Delta::Deleted => "deleted   ",
        Delta::Modified => "modified  ",
        Delta::Renamed => "renamed   ",
        Delta::Copied => "copied    ",
        Delta::Ignored => "ignored   ",
        Delta::Untracked => "untracked ",
        Delta::Typechange => "typechange",
        Delta::Unreadable => "unreadable",
        Delta::Conflicted => "conflicted",
    }
}

fn ui(frame: &mut Frame, state: &State) {
    let lines = collapsed_items_iter(&state.items)
        .map(|(i, item)| (i, item))
        .flat_map(|(i, item)| {
            let mut text = if let Some(ref text) = item.header {
                Line::styled(text, Style::new().fg(Color::Blue))
            } else if let Item {
                diff_line: Some(diff),
                ..
            } = item
            {
                Line::raw(diff)
            } else if let Item {
                diff_hunk: Some(hunk),
                ..
            } = item
            {
                Line::styled(hunk, Style::new().add_modifier(Modifier::REVERSED))
            } else if let Item { oid: Some(oid), .. } = item {
                Line::from(vec![Span::styled(
                    hex::encode(oid.as_bytes()).as_str()[..8].to_string(),
                    Style::new(),
                )])
            } else if let Item {
                file: Some(file),
                status,
                ..
            } = item
            {
                match status {
                    Some(s) => Line::styled(format!("{}   {}", s, file), Style::new()),
                    None => Line::styled(format!("{}", file), Style::new().fg(Color::LightMagenta)),
                }
            } else {
                Line::styled("".to_string(), Style::new())
            };

            text.patch_style(if state.selected == i {
                Style::new().add_modifier(Modifier::BOLD)
            } else {
                Style::new().add_modifier(Modifier::DIM)
            });

            if item.section.is_some_and(|collapsed| collapsed) {
                text.spans.push(Span::raw("…"))
            }

            if item.header.is_some() {
                vec![Line::raw(""), text]
            } else {
                vec![text]
            }
        })
        .collect::<Vec<_>>();

    frame.render_widget(Paragraph::new(Text::from(lines)), frame.size());
}

fn handle_events(state: &mut State, repo: &mut Repository) -> io::Result<bool> {
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => state.quit = true,
                    KeyCode::Char('j') => {
                        state.selected = collapsed_items_iter(&state.items)
                            .find(|(i, item)| i > &state.selected && item.diff_line.is_none())
                            .map(|(i, _item)| i)
                            .unwrap_or(state.selected)
                    }
                    KeyCode::Char('k') => {
                        state.selected = collapsed_items_iter(&state.items)
                            .filter(|(i, item)| i < &state.selected && item.diff_line.is_none())
                            .last()
                            .map(|(i, _item)| i)
                            .unwrap_or(state.selected)
                    }
                    KeyCode::Char('s') => {
                        if let Some(ref file) = state.items[state.selected].file {
                            let index = &mut repo.index().unwrap();
                            index.add_path(Path::new(&file)).unwrap();

                            index.write().unwrap();
                            state.items = create_status_items(repo);
                        }
                    }
                    KeyCode::Char('u') => {
                        if let Some(ref file) = state.items[state.selected].file {
                            let index = &mut repo.index().unwrap();
                            index.remove_path(Path::new(&file)).unwrap();
                            index.write().unwrap();
                            state.items = create_status_items(repo);
                        }
                    }
                    KeyCode::Tab => {
                        if let Some(ref mut collapsed) = state.items[state.selected].section {
                            *collapsed = !*collapsed;
                        };
                    }
                    _ => (),
                }
            }
        }
    }
    Ok(false)
}

fn file_hunk_groups<'a>(items: Vec<Item>) -> BTreeMap<String, BTreeMap<String, Vec<Item>>> {
    use itertools::Itertools;

    items
        .into_iter()
        .group_by(|item| item.file.clone().unwrap())
        .into_iter()
        .map(|(file, file_items)| {
            (
                file,
                file_items
                    .into_iter()
                    .group_by(|item| item.diff_hunk.clone().unwrap())
                    .into_iter()
                    .map(|(hunk, hunk_items)| (hunk, hunk_items.collect()))
                    .collect(),
            )
        })
        .collect()
}

fn collapsed_items_iter<'a>(items: &'a Vec<Item>) -> impl Iterator<Item = (usize, &'a Item)> {
    items
        .iter()
        .enumerate()
        .scan(None, |collapse_depth, (i, next)| {
            if collapse_depth.is_some_and(|depth| depth < next.depth) {
                return Some(None);
            }

            *collapse_depth = next
                .section
                .is_some_and(|collapsed| collapsed)
                .then(|| next.depth);

            Some(Some((i, next)))
        })
        .filter_map(|e| e)
}
