use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = crate::APP_NAME)]
#[command(flatten_help = true)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,
    /// Print one frame and exit. Useful for debugging.
    #[clap(long, action, default_value_t = false)]
    pub print: bool,
    /// Enable logging to 'gitu.log'
    #[clap(long, action, default_value_t = false)]
    pub log: bool,

    #[clap(long, action, default_value_t = false)]
    /// Print version
    pub version: bool,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Show { reference: String },
}
