mod diff;
mod git;
mod process;
mod ui;

use std::{
    collections::HashSet,
    io::{self, stdout, Read, Write},
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
    style::{Color, Style},
    Terminal,
};

#[derive(Debug)]
struct State {
    quit: bool,
    selected: usize,
    items: Vec<Item>,
    collapsed: HashSet<Item>,
    command: Option<IssuedCommand>,
}

impl State {
    fn issue_command(&mut self, input: &[u8], command: Command) -> Result<(), io::Error> {
        if !self.command.as_mut().is_some_and(|cmd| cmd.is_running()) {
            self.command = Some(IssuedCommand::spawn(input, command)?);
        }

        Ok(())
    }

    fn select_next(&mut self) {
        self.selected = collapsed_items_iter(&self.collapsed, &self.items)
            .find(|(i, item)| i > &self.selected && item.diff_line.is_none())
            .map(|(i, _item)| i)
            .unwrap_or(self.selected)
    }

    fn select_previous(&mut self) {
        self.selected = collapsed_items_iter(&self.collapsed, &self.items)
            .filter(|(i, item)| i < &self.selected && item.diff_line.is_none())
            .last()
            .map(|(i, _item)| i)
            .unwrap_or(self.selected)
    }

    fn toggle_section(&mut self) {
        let selected = &self.items[self.selected];

        if selected.section {
            if self.collapsed.contains(selected) {
                self.collapsed.remove(selected);
            } else {
                self.collapsed.insert(selected.clone());
            }
        }
    }

    fn clamp_selected(&mut self) {
        self.selected = self.selected.clamp(0, self.items.len().saturating_sub(1))
    }
}

#[derive(Debug)]
struct IssuedCommand {
    args: String,
    child: Child,
    output: Vec<u8>,
    finish_acked: bool,
}

impl IssuedCommand {
    fn spawn(input: &[u8], mut command: Command) -> Result<IssuedCommand, io::Error> {
        command.stdin(Stdio::piped());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        let mut child = command.spawn()?;

        child
            .stdin
            .take()
            .unwrap_or_else(|| panic!("No stdin for process"))
            .write_all(input)
            .unwrap_or_else(|_| panic!("Error writing to stdin"));

        let issued_command = IssuedCommand {
            args: format_command(&command),
            child,
            output: vec![],
            finish_acked: false,
        };
        Ok(issued_command)
    }

    fn read_command_output_to_buffer(&mut self) {
        if let Some(stderr) = self.child.stderr.as_mut() {
            let mut buffer = [0; 256];

            let read = stderr
                .read(&mut buffer)
                .expect("Error reading child stderr");

            self.output.extend(&buffer[..read]);
        }
    }

    fn is_running(&mut self) -> bool {
        !self.child.try_wait().is_ok_and(|status| status.is_some())
    }

    fn just_finished(&mut self) -> bool {
        if self.finish_acked {
            return false;
        }

        let Some(_status) = self.child.try_wait().expect("Error awaiting child") else {
            return false;
        };

        self.finish_acked = true;
        true
    }
}

#[derive(Default, Clone, Debug, PartialEq, Eq, Hash)]
struct Item {
    display: Option<(String, Style)>,
    section: bool,
    depth: usize,
    delta: Option<Delta>,
    hunk: Option<Hunk>,
    diff_line: Option<String>,
    reference: Option<String>,
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
        if let Some(cmd) = &mut state.command {
            cmd.read_command_output_to_buffer();

            if cmd.just_finished() {
                state.items = create_status_items();
            }
        }

        handle_events(&mut state, &mut terminal)?;
        state.clamp_selected();

        terminal.draw(|frame| ui::ui(frame, &state))?;
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
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
            display: Some((
                format!("{} ({})", header, diff.deltas.len()),
                Style::new().fg(Color::Yellow),
            )),
            section: true,
            depth: 0,
            ..Default::default()
        });
    }

    for delta in diff.deltas {
        let hunk_delta = delta.clone();

        items.push(Item {
            delta: Some(delta.clone()),
            display: Some((
                if delta.old_file == delta.new_file {
                    delta.new_file
                } else {
                    format!("{} -> {}", delta.old_file, delta.new_file)
                },
                Style::new().fg(Color::Yellow),
            )),
            section: true,
            depth: 1,
            ..Default::default()
        });

        for hunk in delta.hunks {
            items.push(Item {
                display: Some((hunk.display_header(), Style::new().fg(Color::Yellow))),
                section: true,
                depth: 2,
                delta: Some(hunk_delta.clone()),
                hunk: Some(hunk.clone()),
                ..Default::default()
            });

            for line in hunk.content_lines() {
                items.push(Item {
                    display: Some((line.colored, Style::new())),
                    depth: 3,
                    delta: Some(hunk_delta.clone()),
                    hunk: Some(hunk.clone()),
                    diff_line: Some(line.plain),
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
        display: Some((header.to_string(), Style::new().fg(Color::Yellow))),
        section: true,
        depth: 0,
        ..Default::default()
    });
    log.lines().for_each(|log_line| {
        items.push(Item {
            display: Some((log_line.to_string(), Style::new())),
            depth: 1,
            reference: Some(
                strip_ansi_escapes::strip_str(log_line)
                    .to_string()
                    .split_whitespace()
                    .next()
                    .expect("Error extracting ref")
                    .to_string(),
            ),
            ..Default::default()
        })
    });
    items
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

fn handle_events<B: Backend>(state: &mut State, terminal: &mut Terminal<B>) -> io::Result<()> {
    if !event::poll(std::time::Duration::from_millis(50))? {
        return Ok(());
    }

    let selected = &state.items[state.selected];

    if let Event::Key(key) = event::read()? {
        if key.kind == event::KeyEventKind::Press {
            match key.code {
                KeyCode::Char('f') => {
                    state.issue_command(&[], git::fetch_all_cmd())?;
                    state.items = create_status_items();
                }
                KeyCode::Char('g') => state.items = create_status_items(),
                KeyCode::Char('q') => state.quit = true,
                KeyCode::Char('j') => state.select_next(),
                KeyCode::Char('k') => state.select_previous(),
                KeyCode::Char('s') => {
                    match selected {
                        Item { hunk: Some(h), .. } => state
                            .issue_command(h.format_patch().as_bytes(), git::stage_patch_cmd())?,
                        Item { delta: Some(d), .. } => {
                            state.issue_command(&[], git::stage_file_cmd(d))?
                        }
                        _ => (),
                    }

                    state.items = create_status_items();
                }
                KeyCode::Char('u') => {
                    match selected {
                        Item { hunk: Some(h), .. } => state
                            .issue_command(h.format_patch().as_bytes(), git::unstage_patch_cmd())?,
                        Item { delta: Some(d), .. } => {
                            state.issue_command(&[], git::unstage_file_cmd(d))?
                        }
                        _ => (),
                    };
                    state.items = create_status_items();
                }
                KeyCode::Char('c') => {
                    open_subscreen(terminal, &[], git::commit_cmd())?;
                    state.items = create_status_items();
                }
                KeyCode::Char('P') => state.issue_command(&[], git::push_cmd())?,
                KeyCode::Char('p') => state.issue_command(&[], git::pull_cmd())?,
                KeyCode::Enter => {
                    match selected {
                        Item {
                            delta: Some(d),
                            hunk: Some(h),
                            ..
                        } => {
                            open_subscreen(terminal, &[], editor_cmd(d, Some(h)))?;
                        }
                        Item { delta: Some(d), .. } => {
                            open_subscreen(terminal, &[], editor_cmd(d, None))?;
                        }
                        Item {
                            reference: Some(r), ..
                        } => {
                            open_subscreen(terminal, &[], git::show_cmd(r))?;
                        }
                        _ => (),
                    };

                    state.items = create_status_items();
                }
                KeyCode::Tab => state.toggle_section(),
                _ => (),
            }
        }
    }

    Ok(())
}

fn editor_cmd(delta: &Delta, maybe_hunk: Option<&Hunk>) -> Command {
    let mut cmd = Command::new("hx");
    cmd.arg(match maybe_hunk {
        Some(hunk) => format!("{}:{}", &delta.new_file, hunk.new_start),
        None => delta.new_file.clone(),
    });
    cmd
}

fn open_subscreen<B: Backend>(
    terminal: &mut Terminal<B>,
    input: &[u8],
    mut cmd: Command,
) -> Result<(), io::Error> {
    crossterm::execute!(stdout(), EnterAlternateScreen)?;

    cmd.stdin(Stdio::piped());
    let mut cmd = cmd.spawn()?;
    cmd.stdin
        .take()
        .expect("Error taking stdin")
        .write_all(input)?;

    cmd.wait()?;

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
