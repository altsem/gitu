mod diff;
mod git;
mod process;
mod screen;
mod ui;

use std::{
    collections::HashSet,
    io::{self, stdout, Read, Write},
    process::{Child, Command, Stdio},
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use diff::{Delta, Hunk};
use ratatui::{
    prelude::{Backend, CrosstermBackend},
    style::{Color, Style},
    Terminal,
};
use screen::Screen;

struct State {
    quit: bool,
    screens: Vec<Screen>,
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
    let mut screens = vec![];

    screens.push(Screen {
        name: "status",
        cursor: 0,
        scroll: 0,
        size: crossterm::terminal::size()?,
        refresh_items: Box::new(create_status_items),
        items: create_status_items(),
        collapsed: HashSet::new(),
        command: None,
    });

    let mut state = State {
        quit: false,
        screens,
    };

    while !state.quit {
        if let Some(screen) = state.screens.last_mut() {
            screen.handle_command_output();
        }

        handle_events(&mut state, &mut terminal)?;

        if let Some(screen) = state.screens.last_mut() {
            screen.clamp_selected();
            terminal.draw(|frame| ui::ui(frame, screen))?;
        }
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

fn create_show_items(reference: &str) -> Vec<Item> {
    let mut items = vec![];
    items.push(Item {
        display: Some((git::show_summary(reference), Style::new())),
        ..Default::default()
    });
    items.extend(create_diff(diff::Diff::parse(&git::show(reference)), 0));
    items
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

    items.extend(create_diff(diff, 1));

    items
}

fn create_diff(diff: diff::Diff, depth: usize) -> Vec<Item> {
    let mut items = vec![];

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
            depth,
            ..Default::default()
        });

        for hunk in delta.hunks {
            items.push(Item {
                display: Some((hunk.display_header(), Style::new().fg(Color::Yellow))),
                section: true,
                depth: depth + 1,
                delta: Some(hunk_delta.clone()),
                hunk: Some(hunk.clone()),
                ..Default::default()
            });

            for line in hunk.content_lines() {
                items.push(Item {
                    display: Some((line.colored, Style::new())),
                    depth: depth + 2,
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

    items.extend(create_log(log));

    items
}

fn create_log(log: &str) -> Vec<Item> {
    let mut items = vec![];

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
            .collect::<Vec<_>>()
            .join(" ")
    );
    command_display
}

fn handle_events<B: Backend>(state: &mut State, terminal: &mut Terminal<B>) -> io::Result<()> {
    if !event::poll(std::time::Duration::from_millis(50))? {
        return Ok(());
    }

    let Some(screen) = state.screens.last_mut() else {
        panic!("No screen");
    };

    let selected = &screen.items[screen.cursor];

    match event::read()? {
        Event::Resize(w, h) => screen.size = (w, h),
        Event::Key(key) => {
            if key.kind == KeyEventKind::Press {
                match (key.modifiers, key.code) {
                    // Generic
                    (KeyModifiers::NONE, KeyCode::Char('q')) => state.quit = true,
                    (KeyModifiers::NONE, KeyCode::Char('g')) => screen.refresh_items(),

                    // Navigation
                    (KeyModifiers::NONE, KeyCode::Tab) => screen.toggle_section(),
                    (KeyModifiers::NONE, KeyCode::Char('k')) => screen.select_previous(),
                    (KeyModifiers::NONE, KeyCode::Char('j')) => screen.select_next(),

                    (KeyModifiers::CONTROL, KeyCode::Char('u')) => screen.scroll_half_page_up(),
                    (KeyModifiers::CONTROL, KeyCode::Char('d')) => screen.scroll_half_page_down(),

                    // Listing / showing
                    (KeyModifiers::NONE, KeyCode::Char('l')) => {
                        goto_log_screen(&mut state.screens)?
                    }

                    (KeyModifiers::NONE, KeyCode::Enter) => match selected {
                        Item {
                            delta: Some(d),
                            hunk: Some(h),
                            ..
                        } => {
                            open_subscreen(terminal, &[], editor_cmd(d, Some(h)))?;
                            screen.refresh_items();
                        }
                        Item { delta: Some(d), .. } => {
                            open_subscreen(terminal, &[], editor_cmd(d, None))?;
                            screen.refresh_items();
                        }
                        Item {
                            reference: Some(r), ..
                        } => {
                            let reference = r.to_string();
                            goto_show_screen(&mut state.screens, reference)?;
                        }
                        _ => (),
                    },

                    // Commands
                    (KeyModifiers::NONE, KeyCode::Char('f')) => {
                        screen.issue_command(&[], git::fetch_all_cmd())?;
                        screen.refresh_items();
                    }
                    (KeyModifiers::NONE, KeyCode::Char('s')) => {
                        match selected {
                            Item { hunk: Some(h), .. } => screen.issue_command(
                                h.format_patch().as_bytes(),
                                git::stage_patch_cmd(),
                            )?,
                            Item { delta: Some(d), .. } => {
                                screen.issue_command(&[], git::stage_file_cmd(d))?
                            }
                            _ => (),
                        }

                        screen.refresh_items();
                    }
                    (KeyModifiers::NONE, KeyCode::Char('u')) => {
                        match selected {
                            Item { hunk: Some(h), .. } => screen.issue_command(
                                h.format_patch().as_bytes(),
                                git::unstage_patch_cmd(),
                            )?,
                            Item { delta: Some(d), .. } => {
                                screen.issue_command(&[], git::unstage_file_cmd(d))?
                            }
                            _ => (),
                        }

                        screen.refresh_items();
                    }
                    (KeyModifiers::NONE, KeyCode::Char('c')) => {
                        open_subscreen(terminal, &[], git::commit_cmd())?;
                        screen.refresh_items();
                    }
                    (KeyModifiers::SHIFT, KeyCode::Char('P')) => {
                        screen.issue_command(&[], git::push_cmd())?
                    }
                    (KeyModifiers::NONE, KeyCode::Char('p')) => {
                        screen.issue_command(&[], git::pull_cmd())?
                    }
                    _ => (),
                }
            }
        }
        _ => (),
    }

    if state.quit {
        state.screens.pop();
        if let Some(screen) = state.screens.last_mut() {
            state.quit = false;
            screen.refresh_items();
        }
    }

    Ok(())
}

fn goto_show_screen(screens: &mut Vec<Screen>, reference: String) -> Result<(), io::Error> {
    let r = reference.clone();

    screens.push(Screen {
        name: "show",
        cursor: 0,
        scroll: 0,
        size: crossterm::terminal::size()?,
        refresh_items: Box::new(move || create_show_items(&r)),
        items: create_show_items(&reference),
        collapsed: HashSet::new(),
        command: None,
    });

    Ok(())
}

fn goto_log_screen(screens: &mut Vec<Screen>) -> Result<(), io::Error> {
    screens.drain(1..);
    screens.push(Screen {
        name: "log",
        cursor: 0,
        scroll: 0,
        size: crossterm::terminal::size()?,
        refresh_items: Box::new(move || create_log(&git::log())),
        items: create_log(&git::log()),
        collapsed: HashSet::new(),
        command: None,
    });

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
