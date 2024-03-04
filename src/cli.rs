use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = crate::APP_NAME)]
#[command(flatten_help = true)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,
    #[clap(long, action, default_value_t = false)]
    pub exit_immediately: bool,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Show { reference: String },
}
