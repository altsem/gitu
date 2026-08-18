use clap::Parser;
use gitu::{
    cli::Args,
    config::{self, Config},
    error::Error,
    term::{self, Term},
    Res,
};
use log::LevelFilter;
use std::{backtrace::Backtrace, fmt::Display, panic, sync::Arc};

fn main() -> std::process::ExitCode {
    let result = start();
    if let Err(e) = &result {
        eprintln!("{e}");
        std::process::ExitCode::FAILURE
    } else {
        std::process::ExitCode::SUCCESS
    }
}

fn start() -> Res<()> {
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

    let config = Arc::new(config::init_config(args.config.clone())?);
    let config_ref = config.clone();

    panic::set_hook(Box::new(move |panic_info| {
        print_err(term::backend().reset_term(&config));

        eprintln!("{}", panic_info);
        eprintln!("trace: \n{}", Backtrace::force_capture());
    }));

    log::debug!("Initializing terminal backend");
    let mut term = term::backend();

    if !args.print {
        term.setup_term(&config_ref).map_err(Error::Term)?;
    }

    let result = setup_term_and_run(&mut term, config_ref.clone(), &args);
    term.reset_term(&config_ref).map_err(Error::Term)?;
    result
}

fn setup_term_and_run(term: &mut Term, config: Arc<Config>, args: &Args) -> Res<()> {
    log::debug!("Starting app");
    gitu::run(config, args, term)
}

fn print_err<T, E: Display>(result: Result<T, E>) {
    match result {
        Ok(_) => (),
        Err(error) => eprintln!("Error: {}", error),
    };
}
