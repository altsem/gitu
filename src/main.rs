use clap::Parser;
use gitu::{cli::Args, error::Error, term, Res};
use log::LevelFilter;
use ratatui::Terminal;

pub fn main() -> Res<()> {
    let args = Args::parse();

    if args.version {
        // Setting cargo_suffix enables falling back to Cargo.toml for version
        // `cargo install --locked gitu` would fail otherwise, as there's no git repo
        println!("gitu {}", git_version::git_version!(cargo_suffix = ""));
        return Ok(());
    }

    if args.log {
        simple_logging::log_to_file(gitu::LOG_FILE_NAME, LevelFilter::Debug)
            .map_err(Error::OpenLogFile)?;
    }

    setup_term_and_run(&args)?;

    Ok(())
}

fn setup_term_and_run(args: &Args) -> Res<()> {
    log::debug!("Initializing terminal backend");
    let mut terminal = Terminal::new(term::backend()).map_err(Error::Term)?;

    // Prevents cursor flash when opening gitu
    terminal.hide_cursor().map_err(Error::Term)?;
    terminal.clear().map_err(Error::Term)?;

    log::debug!("Starting app");
    gitu::run(args, &mut terminal)
}
