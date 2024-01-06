mod diff;

use std::{
    collections::HashSet,
    io::{self, stdout, Write},
    process::{Command, Stdio},
};

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use diff::{Delta, Hunk};
use ratatui::{
    prelude::{Backend, CrosstermBackend},
    style::{Color, Modifier, Style},
    text::Text,
    widgets::Paragraph,
    Frame, Terminal,
};

#[derive(Debug)]
struct State {
    quit: bool,
    selected: usize,
    items: Vec<Item>,
    collapsed: HashSet<Item>,
}

#[derive(Default, Clone, Debug, PartialEq, Eq, Hash)]
struct Item {
    header: Option<String>,
    section: bool,
    depth: usize,
    delta: Option<Delta>,
    hunk: Option<Hunk>,
    line: Option<String>,
}

// TODO Show repo state (repo.state())

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    let items = create_status_items();

    let mut state = State {
        quit: false,
        selected: 0,
        items,
        collapsed: HashSet::new(),
    };

    while !state.quit {
        terminal.draw(|frame| ui(frame, &state))?;
        handle_events(&mut state, &mut terminal)?;
        state.selected = state.selected.clamp(0, state.items.len().saturating_sub(1));
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn create_status_items() -> Vec<Item> {
    let mut items = vec![];

    // TODO items.extend(create_status_section(&repo, None, "Untracked files"));

    items.extend(create_status_section(
        diff::Diff::parse(&pipe(
            run("git", &["diff"]).as_bytes(),
            "delta",
            &["--color-only"],
        )),
        "\nUnstaged changes",
    ));

    items.extend(create_status_section(
        diff::Diff::parse(&pipe(
            run("git", &["diff", "--staged"]).as_bytes(),
            "delta",
            &["--color-only"],
        )),
        "\nStaged changes",
    ));

    items.push(Item {
        header: Some("\nRecent commits".to_string()),
        section: true,
        depth: 0,
        ..Default::default()
    });
    run(
        "git",
        &["log", "-n", "5", "--oneline", "--decorate", "--color"],
    )
    .lines()
    .for_each(|log_line| {
        items.push(Item {
            depth: 1,
            line: Some(log_line.to_string()),
            ..Default::default()
        })
    });

    items
}

fn run(program: &str, args: &[&str]) -> String {
    String::from_utf8(
        Command::new(program)
            .args(args)
            .output()
            .unwrap_or_else(|_| panic!("Couldn't execute '{}'", program))
            .stdout,
    )
    .unwrap()
}

fn pipe(input: &[u8], program: &str, args: &[&str]) -> String {
    let mut command = Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap_or_else(|_| panic!("Error executing '{}'", program));
    command
        .stdin
        .take()
        .unwrap_or_else(|| panic!("No stdin for {} process", program))
        .write_all(input)
        .unwrap_or_else(|_| panic!("Error writing to '{}' stdin", program));
    String::from_utf8(
        command
            .wait_with_output()
            .unwrap_or_else(|_| panic!("Error writing {} output", program))
            .stdout,
    )
    .unwrap()
}

fn create_status_section<'a>(diff: diff::Diff, header: &str) -> Vec<Item> {
    let mut items = vec![];

    if !diff.deltas.is_empty() {
        items.push(Item {
            header: Some(format!("{} ({})", header.to_string(), diff.deltas.len())),
            section: true,
            depth: 0,
            ..Default::default()
        });
    }

    for delta in diff.deltas {
        let hunk_delta = delta.clone();

        items.push(Item {
            delta: Some(delta.clone()),
            header: Some(if delta.old_file == delta.new_file {
                delta.new_file
            } else {
                format!("{} -> {}", delta.old_file, delta.new_file)
            }),
            section: true,
            depth: 1,
            ..Default::default()
        });

        for hunk in delta.hunks {
            items.push(Item {
                header: Some(hunk.display_header()),
                section: true,
                depth: 2,
                delta: Some(hunk_delta.clone()),
                hunk: Some(hunk.clone()),
                ..Default::default()
            });

            for line in hunk.content.lines() {
                items.push(Item {
                    depth: 3,
                    delta: Some(hunk_delta.clone()),
                    hunk: Some(hunk.clone()),
                    line: Some(line.to_string()),
                    ..Default::default()
                });
            }
        }
    }

    items
}

fn ui(frame: &mut Frame, state: &State) {
    let mut highlight_depth = None;

    let lines = collapsed_items_iter(&state.collapsed, &state.items)
        .flat_map(|(i, item)| {
            let mut text = if let Some(ref text) = item.header {
                Text::styled(text, Style::new().fg(Color::Yellow))
            } else if let Item {
                line: Some(line), ..
            } = item
            {
                use ansi_to_tui::IntoText;
                line.into_text().expect("Couldn't read ansi codes")
            } else {
                panic!("Couldn't format item");
            };

            if state.collapsed.contains(&item) {
                text.lines
                    .last_mut()
                    .expect("No last line found")
                    .spans
                    .push("…".into());
            }

            if state.selected == i {
                highlight_depth = Some(item.depth);
            } else if highlight_depth.is_some_and(|hd| hd >= item.depth) {
                highlight_depth = None;
            }

            text.patch_style(if highlight_depth.is_some() {
                Style::new().add_modifier(Modifier::BOLD)
            } else {
                Style::new().add_modifier(Modifier::DIM)
            });

            text
        })
        .collect::<Vec<_>>();

    frame.render_widget(Paragraph::new(lines), frame.size());
}

fn handle_events<B: Backend>(state: &mut State, terminal: &mut Terminal<B>) -> io::Result<bool> {
    let selected = &state.items[state.selected];

    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => state.quit = true,
                    KeyCode::Char('j') => {
                        state.selected = collapsed_items_iter(&state.collapsed, &state.items)
                            .find(|(i, item)| i > &state.selected && item.line.is_none())
                            .map(|(i, _item)| i)
                            .unwrap_or(state.selected)
                    }
                    KeyCode::Char('k') => {
                        state.selected = collapsed_items_iter(&state.collapsed, &state.items)
                            .filter(|(i, item)| i < &state.selected && item.line.is_none())
                            .last()
                            .map(|(i, _item)| i)
                            .unwrap_or(state.selected)
                    }
                    KeyCode::Char('s') => match selected {
                        Item {
                            hunk: Some(ref hunk),
                            ..
                        } => {
                            pipe(
                                hunk.format_patch().as_bytes(),
                                "git",
                                &["apply", "--cached"],
                            );
                            state.items = create_status_items();
                        }
                        Item {
                            delta: Some(ref delta),
                            ..
                        } => {
                            run("git", &["add", &delta.new_file]);
                            state.items = create_status_items();
                        }
                        _ => (),
                    },
                    KeyCode::Char('u') => match selected {
                        Item {
                            hunk: Some(ref hunk),
                            ..
                        } => {
                            pipe(
                                hunk.format_patch().as_bytes(),
                                "git",
                                &["apply", "--cached", "--reverse"],
                            );
                            state.items = create_status_items();
                        }
                        Item {
                            delta: Some(ref delta),
                            ..
                        } => {
                            run("git", &["restore", "--staged", &delta.new_file]);
                            state.items = create_status_items();
                        }
                        _ => (),
                    },
                    KeyCode::Char('c') => {
                        open_subscreen(terminal, Command::new("git").arg("commit"))?;
                        state.items = create_status_items();
                    }
                    KeyCode::Enter => match selected {
                        Item {
                            delta: Some(ref delta),
                            hunk: Some(ref hunk),
                            ..
                        } => {
                            open_subscreen(
                                terminal,
                                Command::new("hx")
                                    .arg(format!("{}:{}", &delta.new_file, hunk.new_start)),
                            )?;
                            state.items = create_status_items();
                        }
                        Item {
                            delta: Some(ref delta),
                            ..
                        } => {
                            open_subscreen(terminal, Command::new("hx").arg(&delta.new_file))?;
                            state.items = create_status_items();
                        }
                        _ => (),
                    },
                    KeyCode::Tab => {
                        try_toggle(&mut state.collapsed, selected);
                    }
                    _ => (),
                }
            }
        }
    }
    Ok(false)
}

fn try_toggle(collapsed: &mut HashSet<Item>, selected: &Item) {
    if selected.section {
        if collapsed.contains(&selected) {
            collapsed.remove(&selected);
        } else {
            collapsed.insert(selected.clone());
        }
    }
}

fn open_subscreen<B: Backend>(
    terminal: &mut Terminal<B>,
    arg: &mut Command,
) -> Result<(), io::Error> {
    crossterm::execute!(stdout(), EnterAlternateScreen)?;
    let mut editor = arg.spawn()?;
    editor.wait()?;
    crossterm::execute!(stdout(), LeaveAlternateScreen)?;
    crossterm::execute!(
        stdout(),
        crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
    )?;
    terminal.clear()?;
    Ok(())
}

fn collapsed_items_iter<'a>(
    collapsed: &'a HashSet<Item>,
    items: &'a Vec<Item>,
) -> impl Iterator<Item = (usize, &'a Item)> {
    items
        .iter()
        .enumerate()
        .scan(None, |collapse_depth, (i, next)| {
            if collapse_depth.is_some_and(|depth| depth < next.depth) {
                return Some(None);
            }

            *collapse_depth = if next.section && collapsed.contains(next) {
                Some(next.depth)
            } else {
                None
            };

            Some(Some((i, next)))
        })
        .flatten()
}
