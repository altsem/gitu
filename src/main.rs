mod command;
mod diff;
mod git;
mod items;
mod process;
mod screen;
mod ui;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{
        self, disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    },
    ExecutableCommand,
};
use diff::{Delta, Hunk};
use items::Item;
use ratatui::{
    prelude::{Backend, CrosstermBackend},
    Terminal,
};
use screen::Screen;
use std::{
    io::{self, stdout},
    process::{Command, Stdio},
};

struct State {
    quit: bool,
    screens: Vec<Screen>,
}

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;

    run_app(Terminal::new(CrosstermBackend::new(stdout()))?)?;

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

fn run_app(mut terminal: Terminal<CrosstermBackend<io::Stdout>>) -> Result<(), io::Error> {
    let mut state = State {
        quit: false,
        screens: vec![screen::status::create(terminal::size()?)],
    };

    while !state.quit {
        if let Some(screen) = state.screens.last_mut() {
            terminal.draw(|frame| ui::ui(frame, screen))?;
            screen.handle_command_output();
        }

        handle_events(&mut state, &mut terminal)?;

        if let Some(screen) = state.screens.last_mut() {
            screen.clamp_cursor();
        }
    }

    Ok(())
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
                            goto_show_screen(r.clone(), &mut state.screens)?;
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

fn goto_show_screen(reference: String, screens: &mut Vec<Screen>) -> Result<(), io::Error> {
    screens.push(screen::show::create(terminal::size()?, reference));
    Ok(())
}

fn goto_log_screen(screens: &mut Vec<Screen>) -> Result<(), io::Error> {
    screens.drain(1..);
    screens.push(screen::log::create(terminal::size()?));
    Ok(())
}

fn editor_cmd(delta: &Delta, maybe_hunk: Option<&Hunk>) -> Command {
    let editor = std::env::var("EDITOR").expect("EDITOR not set");
    let mut cmd = Command::new(editor.clone());
    let args = match maybe_hunk {
        Some(hunk) => match editor.as_str() {
            "vi" | "vim" | "nvim" | "nano" => {
                vec![format!("+{}", hunk.new_start), delta.new_file.clone()]
            }
            _ => vec![format!("{}:{}", &delta.new_file, hunk.new_start)],
        },
        None => vec![delta.new_file.clone()],
    };

    cmd.args(args);
    cmd
}

pub(crate) fn open_subscreen<B: Backend>(
    terminal: &mut Terminal<B>,
    input: &[u8],
    mut cmd: Command,
) -> Result<(), io::Error> {
    crossterm::execute!(stdout(), EnterAlternateScreen)?;

    cmd.stdin(Stdio::piped());
    let mut cmd = cmd.spawn()?;

    use std::io::Write;
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
