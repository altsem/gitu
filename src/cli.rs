use clap::{Parser, Subcommand};

#[derive(Default, Debug, Parser)]
#[command(name = crate::APP_NAME)]
#[command(flatten_help = true)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,
    /// Print one frame and exit. Useful for debugging.
    #[clap(long, action)]
    pub print: bool,
    /// Enable logging to 'gitu.log'
    #[clap(long, action)]
    pub log: bool,

    #[clap(long, action)]
    /// Print version
    pub version: bool,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Show { reference: String },
}
