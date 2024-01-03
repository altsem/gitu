mod diff;

use std::{
    io::{self, stdout, Write},
    path::Path, process::{Command, Stdio},
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
        diff::Diff::parse(&git(&["diff"])),
        "Unstaged changes",
    ));

    items.extend(create_status_section(
        diff::Diff::parse(&git(&["diff", "--staged"])),
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

fn git(args: &[&'static str]) -> String {
    String::from_utf8(Command::new("git").args(args).output().expect("Couldn't execute 'git'").stdout).unwrap()
}

fn pipe_git(input: &[u8], args: &[&'static str]) -> String {
    let mut git = Command::new("git").args(args).stdin(Stdio::piped()).spawn().expect("Error executing 'git'");
    git.stdin.take().expect("No stdin for git process").write_all(input).expect("Error writing to git stdin");
    String::from_utf8(git.wait_with_output().expect("Error writing git output").stdout).unwrap()
}

fn create_status_section<'a>(diff: diff::Diff, header: &str) -> Vec<Item> {
    let mut items = vec![];

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
                            let index = &mut repo.index().unwrap();
                            index.add_path(Path::new(&delta.new_file)).unwrap();

                            index.write().unwrap();
                            state.items = create_status_items(repo);
                        }
                        Item {
                            hunk: Some(ref hunk),
                            ..
                        } => {
                            pipe_git(hunk.format_patch().as_bytes(), &["apply", "--cached"]);
                            state.items = create_status_items(repo);
                        }
                        // TODO Stage lines
                        _ => panic!("Couldn't stage"),
                    },
                    KeyCode::Char('u') => {
                        match state.items[state.selected] {
                            Item {
                                delta: Some(ref delta),
                                ..
                            } => {
                                let index = &mut repo.index().unwrap();
                                index.remove_path(Path::new(&delta.new_file)).unwrap();
                                index.write().unwrap();
                                state.items = create_status_items(repo);
                            }
                            Item {
                                hunk: Some(ref hunk),
                                ..
                            } => {
                                pipe_git(hunk.format_patch().as_bytes(), &["apply", "--cached", "--reverse"]);
                                state.items = create_status_items(repo);
                            }
                            // TODO Stage lines
                            _ => panic!("Couldn't unstage"),
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
