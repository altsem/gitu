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

    let mut terminal = Terminal::new(term::create_backend()?).map_err(Error::Term)?;

    if args.print {
        return gitu::run(&args, &mut terminal);
    }

    terminal.backend_mut().enter_alternate_screen()?;
    terminal.backend_mut().enable_raw_mode()?;

    // Prevents cursor flash when opening gitu
    terminal.hide_cursor().map_err(Error::Term)?;
    terminal.clear().map_err(Error::Term)?;

    gitu::run(&args, &mut terminal)?;

    Ok(())
}
