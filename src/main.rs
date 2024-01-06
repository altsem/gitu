mod diff;
mod git;
mod process;

use std::{
    collections::HashSet,
    io::{self, stdout, Read},
    process::{Child, Command, Stdio},
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
    command: Option<IssuedCommand>,
}

#[derive(Debug)]
struct IssuedCommand {
    args: String,
    child: Child,
    output: Vec<u8>,
}

impl IssuedCommand {
    fn spawn(mut command: Command) -> Result<Option<IssuedCommand>, io::Error> {
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
        let issued_command = Some(IssuedCommand {
            args: format_command(&command),
            child: command.spawn()?,
            output: vec![],
        });
        Ok(issued_command)
    }
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
        command: None,
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
        "\nUnstaged changes",
        diff::Diff::parse(&git::diff_unstaged()),
    ));

    items.extend(create_status_section(
        "\nStaged changes",
        diff::Diff::parse(&git::diff_staged()),
    ));

    items.extend(create_log_section("\nRecent commits", &git::log_recent()));

    items
}

fn create_status_section<'a>(header: &str, diff: diff::Diff) -> Vec<Item> {
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

fn create_log_section(header: &str, log: &str) -> Vec<Item> {
    let mut items = vec![];
    items.push(Item {
        header: Some(header.to_string()),
        section: true,
        depth: 0,
        ..Default::default()
    });
    log.lines().for_each(|log_line| {
        items.push(Item {
            depth: 1,
            line: Some(log_line.to_string()),
            ..Default::default()
        })
    });
    items
}

fn ui(frame: &mut Frame, state: &State) {
    let mut highlight_depth = None;

    let mut lines = collapsed_items_iter(&state.collapsed, &state.items)
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
                Style::new()
            } else {
                Style::new().add_modifier(Modifier::DIM)
            });

            text
        })
        .collect::<Vec<_>>();

    if let Some(ref cmd) = state.command {
        lines.extend(Text::from("\n".to_string() + &cmd.args.clone()).lines);
        lines.extend(
            Text::raw(
                String::from_utf8(cmd.output.clone())
                    .expect("Error turning command output to String"),
            )
            .lines,
        );
    }

    frame.render_widget(Paragraph::new(lines), frame.size());
}

fn format_command(cmd: &Command) -> String {
    let command_display = format!(
        "{} {}",
        cmd.get_program().to_string_lossy(),
        cmd.get_args()
            .map(|arg| arg.to_string_lossy())
            .collect::<String>()
    );
    command_display
}

fn handle_events<B: Backend>(state: &mut State, terminal: &mut Terminal<B>) -> io::Result<bool> {
    let selected = &state.items[state.selected];

    if let Some(ref mut cmd) = state.command {
        if let Some(stderr) = cmd.child.stderr.as_mut() {
            let mut buffer = [0; 256];

            let read = stderr
                .read(&mut buffer)
                .expect("Error reading child stderr");

            cmd.output.extend(&buffer[..read]);
        }
    }

    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => state.quit = true,
                    KeyCode::Char('j') => select_next(state),
                    KeyCode::Char('k') => select_previous(state),
                    KeyCode::Char('s') => {
                        match selected {
                            Item { hunk: Some(h), .. } => git::stage_hunk(h),
                            Item { delta: Some(d), .. } => git::stage_file(d),
                            _ => (),
                        }

                        state.items = create_status_items();
                    }
                    KeyCode::Char('u') => {
                        match selected {
                            Item { hunk: Some(h), .. } => git::unstage_hunk(h),
                            Item { delta: Some(d), .. } => git::unstage_file(d),
                            _ => (),
                        }

                        state.items = create_status_items();
                    }
                    KeyCode::Char('c') => {
                        open_subscreen(terminal, git::commit_cmd())?;
                        state.items = create_status_items();
                    }
                    KeyCode::Char('P') => {
                        state.command = IssuedCommand::spawn(git::push_cmd())?;
                        state.items = create_status_items();
                    }
                    KeyCode::Char('p') => {
                        state.command = IssuedCommand::spawn(git::pull_cmd())?;
                        state.items = create_status_items();
                    }
                    KeyCode::Enter => {
                        match selected {
                            Item {
                                delta: Some(d),
                                hunk: Some(h),
                                ..
                            } => {
                                open_subscreen(terminal, editor_cmd(d, Some(h)))?;
                            }
                            Item { delta: Some(d), .. } => {
                                open_subscreen(terminal, editor_cmd(d, None))?;
                            }
                            _ => (),
                        }
                        state.items = create_status_items();
                    }
                    KeyCode::Tab => {
                        try_toggle(&mut state.collapsed, selected);
                    }
                    _ => (),
                }
            }
        }
    }

    if let Some(ref mut cmd) = state.command {
        if let Some(_status) = cmd.child.try_wait().expect("Error awaiting child") {
            state.items = create_status_items();
        }
    }

    Ok(false)
}

fn select_next(state: &mut State) {
    state.selected = collapsed_items_iter(&state.collapsed, &state.items)
        .find(|(i, item)| i > &state.selected && item.line.is_none())
        .map(|(i, _item)| i)
        .unwrap_or(state.selected)
}

fn select_previous(state: &mut State) {
    state.selected = collapsed_items_iter(&state.collapsed, &state.items)
        .filter(|(i, item)| i < &state.selected && item.line.is_none())
        .last()
        .map(|(i, _item)| i)
        .unwrap_or(state.selected)
}

fn editor_cmd(delta: &Delta, maybe_hunk: Option<&Hunk>) -> Command {
    let mut cmd = Command::new("hx");
    cmd.arg(match maybe_hunk {
        Some(hunk) => format!("{}:{}", &delta.new_file, hunk.new_start),
        None => delta.new_file.clone(),
    });
    cmd
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
    mut cmd: Command,
) -> Result<(), io::Error> {
    crossterm::execute!(stdout(), EnterAlternateScreen)?;
    let mut editor = cmd.spawn()?;
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
