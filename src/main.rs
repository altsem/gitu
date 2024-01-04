mod diff;

use std::{
    io::{self, stdout, Write},
    process::{Command, Stdio}, rc::Rc,
};

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use diff::{Delta, Hunk};
use ratatui::{
    prelude::{CrosstermBackend, Backend},
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
    let items = create_status_items();

    let mut state = State {
        quit: false,
        selected: 0,
        items,
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
        diff::Diff::parse(&pipe(run("git", &["diff"]).as_bytes(), "delta", &["--color-only"])),
        "Unstaged changes",
    ));

    items.extend(create_status_section(
        diff::Diff::parse(&pipe(run("git", &["diff", "--staged"]).as_bytes(), "delta", &["--color-only"])),
        "Staged changes",
    ));

    items
}

fn run(program: &str, args: &[&str]) -> String {
    String::from_utf8(Command::new(program).args(args).output().expect(&format!("Couldn't execute '{}'", program)).stdout).unwrap()
}

fn pipe(input: &[u8], program: &str, args: &[&str]) -> String {
    let mut command = Command::new(program).args(args).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().expect(&format!("Error executing '{}'", program));
    command.stdin.take().expect(&format!("No stdin for {} process", program)).write_all(input).expect(&format!("Error writing to '{}' stdin", program));
    String::from_utf8(command.wait_with_output().expect(&format!("Error writing {} output", program)).stdout).unwrap()
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
        let hunk_delta = delta.clone();

        items.push(Item {
            delta: Some(delta.clone()),
            depth: 1,
            header: Some(if delta.old_file == delta.new_file { delta.new_file } else { format!("{} -> {}", delta.old_file, delta.new_file) }),
            section: Some(false),
            ..Default::default()
        });

        for hunk in delta.hunks {
            items.push(Item {
                delta: Some(hunk_delta.clone()),
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
    let mut output = Text::raw("");
    let mut highlight_depth = None;

    collapsed_items_iter(&state.items)
        .map(|(i, item)| (i, item))
        .for_each(|(i, item)| {
            let mut item_text = if let Some(ref text) = item.header {
                Text::styled(text, Style::new().fg(Color::Blue))
            } else if let Item {
                line: Some(diff), ..
            } = item
            {
                use ansi_to_tui::IntoText;
                diff.into_text().expect("Couldn't read ansi codes")
            } else if let Item {
                line: Some(hunk), ..
            } = item
            {
                Text::styled(hunk, Style::new().add_modifier(Modifier::REVERSED))
            } else if let Item {
                file: Some(file),
                status,
                ..
            } = item
            {
                match status {
                    Some(s) => Text::styled(format!("{}   {}", s, file), Style::new()),
                    None => Text::styled(format!("{}", file), Style::new().fg(Color::LightMagenta)),
                }
            } else {
                Text::styled("".to_string(), Style::new())
            };

            if item.section.is_some_and(|collapsed| collapsed) {
                item_text.extend(["…"]);
            }

            if state.selected == i {
                highlight_depth = Some(item.depth);
            } else if highlight_depth.is_some_and(|hd| hd >= item.depth) {
                highlight_depth = None;
            }

            item_text.patch_style(if highlight_depth.is_some() {
                Style::new().add_modifier(Modifier::BOLD)
            } else {
                Style::new().add_modifier(Modifier::DIM)
            });

            output.extend(item_text);
        });

    frame.render_widget(Paragraph::new(output), frame.size());
}

fn handle_events<B: Backend>(state: &mut State, terminal: &mut Terminal<B>) -> io::Result<bool> {
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => state.quit = true,
                    KeyCode::Char('j') => {
                        state.selected = collapsed_items_iter(&state.items)
                            .find(|(i, item)| i > &state.selected && item.line.is_none())
                            .map(|(i, _item)| i)
                            .unwrap_or(state.selected)
                    }
                    KeyCode::Char('k') => {
                        state.selected = collapsed_items_iter(&state.items)
                            .filter(|(i, item)| i < &state.selected && item.line.is_none())
                            .last()
                            .map(|(i, _item)| i)
                            .unwrap_or(state.selected)
                    }
                    KeyCode::Char('s') => match state.items[state.selected] {
                        Item {
                            hunk: Some(ref hunk),
                            ..
                        } => {
                            pipe(hunk.format_patch().as_bytes(), "git", &["apply", "--cached"]);
                            state.items = create_status_items();
                        }
                        Item {
                            delta: Some(ref delta),
                            ..
                        } => {
                            run("git", &["add", &delta.new_file]);
                            state.items = create_status_items();
                        }
                        _ => ()
                    },
                    KeyCode::Char('u') => {
                        match state.items[state.selected] {
                            Item {
                                hunk: Some(ref hunk),
                                ..
                            } => {
                                pipe(hunk.format_patch().as_bytes(), "git", &["apply", "--cached", "--reverse"]);
                                state.items = create_status_items();
                            }
                            Item {
                                delta: Some(ref delta),
                                ..
                            } => {
                                run("git", &["restore", "--staged", &delta.new_file]);
                                state.items = create_status_items();
                            }
                            _ => ()
                        }
                    }
                    KeyCode::Char('c') => {
                        open_subscreen(terminal, Command::new("git").arg("commit"))?;
                        state.items = create_status_items();
                    }
                    KeyCode::Enter => {
                        match state.items[state.selected] {
                            Item {
                                delta: Some(ref delta),
                                hunk: Some(ref hunk),
                                ..
                            } => {
                                open_subscreen(terminal, Command::new("hx").arg(format!("{}:{}", &delta.new_file, hunk.new_start)))?;
                                state.items = create_status_items();
                            }
                            Item {
                                delta: Some(ref delta),
                                ..
                            } => {
                                open_subscreen(terminal, Command::new("hx").arg(&delta.new_file))?;
                                state.items = create_status_items();
                            }
                            _ => ()
                            
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

fn open_subscreen<B: Backend>(terminal: &mut Terminal<B>, arg: &mut Command) -> Result<(), io::Error> {
    crossterm::execute!(stdout(), EnterAlternateScreen)?;
    let mut editor = arg
        .spawn()?;
    editor.wait()?;
    crossterm::execute!(stdout(), LeaveAlternateScreen)?;
    crossterm::execute!(stdout(), crossterm::terminal::Clear(crossterm::terminal::ClearType::All))?;
    terminal.clear()?;
    Ok(())
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
