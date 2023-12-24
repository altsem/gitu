mod diff;

use std::{
    io::{self, stdout},
    path::Path,
};

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use diff::{Delta, Hunk};
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
    oid: Option<git2::Oid>,
    header: Option<String>,
    status: Option<String>,
    delta: Option<Delta>,
    hunk: Option<Hunk>,
    line: Option<String>,
    section: Option<bool>,
}

// TODO Show repo state (repo.state())

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    let mut repo = git2::Repository::open(".").unwrap();
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

fn create_status_items(repo: &git2::Repository) -> Vec<Item> {
    let mut items = vec![];

    // TODO items.extend(create_status_section(&repo, None, "Untracked files"));

    items.extend(create_status_section(
        repo.diff_index_to_workdir(None, None).unwrap(),
        "Unstaged changes",
    ));

    items.extend(create_status_section(
        repo.diff_tree_to_index(
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

fn create_status_section<'a>(diff: git2::Diff, header: &str) -> Vec<Item> {
    let mut items = vec![];

    let diff = diff::Diff::from(diff);

    items.push(Item {
        depth: 0,
        header: Some(header.to_string()),
        section: Some(false),
        ..Default::default()
    });

    for delta in diff.deltas {
        items.push(Item {
            delta: Some(delta.clone()),
            depth: 1,
            header: Some(delta.file_header),
            section: Some(false),
            ..Default::default()
        });

        for hunk in delta.hunks {
            items.push(Item {
                hunk: Some(hunk.clone()),
                depth: 2,
                header: Some(hunk.header()),
                section: Some(false),
                ..Default::default()
            });

            for line in hunk.content.lines() {
                items.push(Item {
                    depth: 3,
                    line: Some(line.to_string()),
                    ..Default::default()
                });
            }
        }
    }

    items
}

fn format_delta_status(delta: &git2::Delta) -> &'_ str {
    match delta {
        git2::Delta::Unmodified => "unmodified",
        git2::Delta::Added => "added     ",
        git2::Delta::Deleted => "deleted   ",
        git2::Delta::Modified => "modified  ",
        git2::Delta::Renamed => "renamed   ",
        git2::Delta::Copied => "copied    ",
        git2::Delta::Ignored => "ignored   ",
        git2::Delta::Untracked => "untracked ",
        git2::Delta::Typechange => "typechange",
        git2::Delta::Unreadable => "unreadable",
        git2::Delta::Conflicted => "conflicted",
    }
}

fn ui(frame: &mut Frame, state: &State) {
    let lines = collapsed_items_iter(&state.items)
        .map(|(i, item)| (i, item))
        .flat_map(|(i, item)| {
            let mut text = if let Some(ref text) = item.header {
                Line::styled(text, Style::new().fg(Color::Blue))
            } else if let Item {
                line: Some(diff), ..
            } = item
            {
                Line::raw(diff)
            } else if let Item {
                line: Some(hunk), ..
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

            vec![text]
        })
        .collect::<Vec<_>>();

    frame.render_widget(Paragraph::new(Text::from(lines)), frame.size());
}

fn handle_events(state: &mut State, repo: &mut git2::Repository) -> io::Result<bool> {
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => state.quit = true,
                    KeyCode::Char('j') => {
                        state.selected = collapsed_items_iter(&state.items)
                            .find(|(i, item)| i > &state.selected)
                            .map(|(i, _item)| i)
                            .unwrap_or(state.selected)
                    }
                    KeyCode::Char('k') => {
                        state.selected = collapsed_items_iter(&state.items)
                            .filter(|(i, item)| i < &state.selected)
                            .last()
                            .map(|(i, _item)| i)
                            .unwrap_or(state.selected)
                    }
                    KeyCode::Char('s') => match state.items[state.selected] {
                        Item {
                            delta: Some(ref delta),
                            ..
                        } => {
                            // let index = &mut repo.index().unwrap();
                            // index.add_path(Path::new(&)).unwrap();

                            // index.write().unwrap();
                            // state.items = create_status_items(repo);
                        }
                        Item {
                            hunk: Some(ref hunk),
                            ..
                        } => {
                            repo.apply(
                                &git2::Diff::from_buffer(hunk.format_patch().as_bytes())
                                    .expect("Couldn't create patch from buffer"),
                                git2::ApplyLocation::Index,
                                None,
                            )
                            .expect("Couldn't apply patch");
                            state.items = create_status_items(repo);
                        }
                        _ => panic!("Couldn't stage"),
                    },
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
