use crate::Res;
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::enable_raw_mode;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;
use crossterm::ExecutableCommand;
use ratatui::backend::Backend;
use ratatui::backend::CrosstermBackend;
use std::fmt::Display;
use std::io::stderr;

pub fn backend() -> impl Backend {
    CrosstermBackend::new(stderr())
}

pub fn enter_alternate_screen() -> Res<()> {
    stderr().execute(EnterAlternateScreen)?;
    Ok(())
}

pub fn alternate_screen<T, F: Fn() -> Res<T>>(fun: F) -> Res<T> {
    stderr().execute(EnterAlternateScreen)?;
    let result = fun();
    stderr().execute(LeaveAlternateScreen)?;
    result
}

pub fn raw_mode<T, F: Fn() -> Res<T>>(fun: F) -> Res<T> {
    enable_raw_mode()?;
    let result = fun();
    disable_raw_mode()?;
    result
}

pub fn cleanup_alternate_screen() {
    print_err(stderr().execute(LeaveAlternateScreen));
}

pub fn cleanup_raw_mode() {
    print_err(disable_raw_mode());
}

fn print_err<T, E: Display>(result: Result<T, E>) {
    match result {
        Ok(_) => (),
        Err(error) => eprintln!("Error: {}", error),
    };
}
