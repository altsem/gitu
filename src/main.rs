use std::{
    error::Error,
    io::{stderr, BufWriter},
};

use clap::Parser;
use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{prelude::CrosstermBackend, Terminal};

pub fn main() -> Result<(), Box<dyn Error>> {
    let mut terminal = Terminal::new(CrosstermBackend::new(BufWriter::new(stderr())))?;
    terminal.hide_cursor()?;
    enable_raw_mode()?;
    stderr().execute(EnterAlternateScreen)?;

    let result = gitu::run(gitu::cli::Args::parse(), &mut terminal);

    stderr().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    result?;

    Ok(())
}
