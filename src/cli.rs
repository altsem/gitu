use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = env!("CARGO_CRATE_NAME"))]
#[command(flatten_help = true)]
pub(crate) struct Args {
    #[command(subcommand)]
    pub(crate) command: Option<Commands>,
    pub(crate) status: bool,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    Show { git_show_args: Vec<String> },
    Log { git_log_args: Vec<String> },
    Diff { git_diff_args: Vec<String> },
}
