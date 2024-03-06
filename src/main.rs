use clap::Parser;
use gitu::{cli::Args, term, Res};
use log::LevelFilter;
use ratatui::Terminal;
use signal_hook::{consts::SIGTERM, iterator::Signals};
use std::{backtrace::Backtrace, panic, process, thread};

pub fn main() -> Res<()> {
    let args = Args::parse();

    if args.log {
        simple_logging::log_to_file("gitu.log", LevelFilter::Trace)?;
    }

    setup_signal_handler()?;

    panic::set_hook(Box::new(|panic_info| {
        term::cleanup_alternate_screen();
        term::cleanup_raw_mode();

        eprintln!("{}", panic_info);
        eprintln!("trace: \n{}", Backtrace::force_capture());
    }));

    if args.print {
        setup_term_and_run(&args)?;
    } else {
        term::alternate_screen(|| term::raw_mode(|| setup_term_and_run(&args)))?
    }

    Ok(())
}

fn setup_signal_handler() -> Res<()> {
    log::debug!("Setting up signal handlers");
    let mut signals = Signals::new([SIGTERM])?;

    thread::spawn(move || {
        for sig in signals.forever() {
            if let SIGTERM = sig {
                let mut terminal = Terminal::new(term::backend()).unwrap();
                terminal.show_cursor().unwrap();
                process::exit(sig);
            };
        }
    });
    Ok(())
}

fn setup_term_and_run(args: &Args) -> Res<()> {
    log::debug!("Initializing terminal backend");
    let mut terminal = Terminal::new(term::backend())?;

    // Prevents cursor flash when opening gitu
    terminal.hide_cursor()?;

    log::debug!("Starting app");
    gitu::run(args, &mut terminal)
}
