use std::{
    error::Error,
    io::{stderr, BufWriter},
    process, thread,
};

use clap::Parser;
use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use log::LevelFilter;
use ratatui::{prelude::CrosstermBackend, Terminal};
use signal_hook::{consts::SIGTERM, iterator::Signals};

pub fn main() -> Result<(), Box<dyn Error>> {
    let args = gitu::cli::Args::parse();

    if args.log {
        simple_logging::log_to_file("gitu.log", LevelFilter::Trace)?;
    }

    log::debug!("Setting up signal handlers");
    setup_signal_handler()?;

    log::debug!("Initializing terminal backend");
    let mut terminal = Terminal::new(CrosstermBackend::new(BufWriter::new(stderr())))?;

    enable_raw_mode()?;
    stderr().execute(EnterAlternateScreen)?;

    log::debug!("Starting app");
    let result = gitu::run(&args, &mut terminal);

    stderr().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    result?;

    Ok(())
}

fn setup_signal_handler() -> Result<(), Box<dyn Error>> {
    let mut signals = Signals::new([SIGTERM])?;
    thread::spawn(move || {
        for sig in signals.forever() {
            if let SIGTERM = sig {
                let mut terminal = Terminal::new(CrosstermBackend::new(stderr())).unwrap();
                terminal.show_cursor().unwrap();
                process::exit(sig);
            };
        }
    });
    Ok(())
}
