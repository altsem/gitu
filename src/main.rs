use std::{
    env::Args,
    io::{self, stdout},
    iter,
    path::Path,
};

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use git2::{DiffDelta, DiffHunk, DiffLine, DiffOptions, Oid, Repository, Status};
use ratatui::{
    prelude::CrosstermBackend,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::Paragraph,
    Frame, Terminal,
};

#[derive(Debug)]
struct State {
    quit: bool,
    selected: usize,
    items: Vec<Item>,
}

#[derive(Default, Clone, Debug)]
struct Item {
    file: Option<String>,
    oid: Option<Oid>,
    header: Option<String>,
    section: Option<Section>,
    status: Option<String>,
    diff_line: Option<String>,
}

#[derive(Clone, Debug)]
struct Section {
    collapsed: bool,
    size: usize,
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

    items.extend(create_status_section(
        &repo,
        "Untracked files",
        Status::is_wt_new,
        |_| None,
    ));

    items.extend(create_status_section(
        &repo,
        "Unstaged changes",
        Status::is_wt_modified,
        unstaged_entry_status,
    ));

    items.extend(create_status_section(
        &repo,
        "Staged changes",
        |status| {
            status.intersects(
                Status::INDEX_NEW
                    | Status::INDEX_DELETED
                    | Status::INDEX_TYPECHANGE
                    | Status::INDEX_RENAMED
                    | Status::INDEX_MODIFIED,
            )
        },
        staged_entry_status,
    ));

    let mut revwalk = repo.revwalk().unwrap();
    revwalk.push_head().unwrap();

    let recent_commits = revwalk
        .take(5)
        .map(|x| Item {
            oid: Some(x.unwrap()),
            ..Default::default()
        })
        .collect::<Vec<_>>();
    if !items.is_empty() {
        items.push(Item {
            header: Some("Recent commits".to_string()),
            section: Some(Section {
                collapsed: false,
                size: recent_commits.len(),
            }),
            ..Default::default()
        });
        items.extend(recent_commits);
    }

    items
}

fn create_status_section(
    repo: &git2::Repository,
    header: &str,
    predicate: impl Fn(&Status) -> bool,
    entry_status: impl Fn(&Status) -> Option<String>,
) -> Vec<Item> {
    let items = repo
        .statuses(None)
        .unwrap()
        .into_iter()
        .filter(|entry| predicate(&entry.status()))
        .map(|entry| Item {
            file: entry.path().map(|value| value.to_string()),
            status: entry_status(&entry.status()),
            ..Default::default()
        })
        .collect::<Vec<_>>();

    if items.is_empty() {
        return vec![];
    }

    let items_count = items.len();

    let mut section_items = items
        .into_iter()
        .rev()
        .flat_map(|mut item| {
            let file = item.file.clone().unwrap();

            let mut diff_options = DiffOptions::new();
            let opts = diff_options.pathspec(file);
            let mut output = String::new();

            let print_diff_line =
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
                };

            // TODO Pass in the diff deltas instead?
            if false {
                repo.diff_tree_to_index(
                    Some(&repo.head().unwrap().peel_to_tree().unwrap()),
                    None,
                    Some(opts),
                )
                .unwrap()
            } else {
                repo.diff_index_to_workdir(None, Some(opts)).unwrap()
            }
            .print(git2::DiffFormat::Patch, print_diff_line)
            .unwrap();

            let diff_line_count = output.lines().count();
            item.section = Some(Section {
                collapsed: true,
                size: diff_line_count,
            });

            output
                .lines()
                .rev()
                .map(|line| Item {
                    diff_line: Some(line.to_string()),
                    ..Default::default()
                })
                .chain(iter::once(item))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let section_items_count = section_items.len();
    section_items.push(Item {
        header: Some(format!("{} ({})", header, items_count)),
        section: Some(Section {
            collapsed: false,
            size: section_items_count,
        }),
        ..Default::default()
    });

    section_items.reverse();

    section_items
}

fn print_diff_line(_delta: DiffDelta, _hunk: Option<DiffHunk>, line: DiffLine) -> bool {
    format!(
        "{}{}",
        match line.origin() {
            '+' | '-' | ' ' => line.origin().to_string(),
            _ => "".to_string(),
        },
        std::str::from_utf8(line.content()).unwrap()
    );
    true
}

fn unstaged_entry_status(status: &Status) -> Option<String> {
    Some(if status.is_wt_modified() {
        "modified".to_string()
    } else {
        format!("{:?}", status)
    })
}

fn staged_entry_status(status: &Status) -> Option<String> {
    Some(if status.is_index_new() {
        "new file".to_string()
    } else if status.is_index_modified() {
        "modified".to_string()
    } else {
        format!("{:?}", status)
    })
}

fn ui(frame: &mut Frame, state: &State) {
    let lines = collapsed_items_iter(&state.items)
        .filter_map(|(i, item)| item.map(|item| (i, item)))
        .flat_map(|(i, item)| {
            let mut text = if let Some(ref text) = item.header {
                Line::styled(text, Style::new().fg(Color::Blue))
            } else if let Item {
                diff_line: Some(diff),
                ..
            } = item
            {
                Line::raw(diff)
            } else if let Item { oid: Some(oid), .. } = item {
                Line::from(vec![Span::styled(
                    hex::encode(oid.as_bytes()).as_str()[..8].to_string(),
                    Style::new().fg(Color::DarkGray),
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

            text.patch_style(
                Style::new()
                    .bg(if state.selected == i {
                        Color::Rgb(20, 60, 80)
                    } else {
                        Color::default()
                    })
                    .add_modifier(Modifier::BOLD),
            );

            if item.section.clone().is_some_and(|s| s.collapsed) {
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
                            .find(|(i, item)| i > &state.selected && item.is_some())
                            .unwrap_or((state.selected, None))
                            .0
                    }
                    KeyCode::Char('k') => {
                        state.selected = collapsed_items_iter(&state.items)
                            .filter(|(i, item)| i < &state.selected && item.is_some())
                            .last()
                            .unwrap_or((state.selected, None))
                            .0
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
                        if let Some(ref mut section) = state.items[state.selected].section {
                            section.collapsed = !section.collapsed;
                        }
                    }
                    _ => (),
                }
            }
        }
    }
    Ok(false)
}

fn collapsed_items_iter<'a>(
    items: &'a Vec<Item>,
) -> impl Iterator<Item = (usize, Option<&'a Item>)> {
    items.iter().enumerate().scan(0, |skips, (i, next)| {
        let next_result = if *skips > 0 {
            *skips -= 1;
            (i, None)
        } else {
            if let Some(Section {
                collapsed: true,
                size,
            }) = next.section
            {
                *skips = size;
            }
            (i, Some(next))
        };

        Some(next_result)
    })
}
