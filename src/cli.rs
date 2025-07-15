use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Default, Debug, Parser)]
#[command(name = "gitu")]
#[command(flatten_help = true)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Send keys on startup (eg: `gitu -k ll`).
    ///     It is possible to send:
    ///     - single char-keys: a, b, c, ...
    ///     - special keys: <backspace>, <enter>, <up>, <tab>, <delete>, <esc>, ...
    ///     - modifiers: <ctrl+a>, <ctrl+shift+alt+a>, <shift+delete>
    #[clap(short, long, verbatim_doc_comment)]
    pub keys: Option<String>,

    /// Print one frame and exit. Useful for debugging.
    #[clap(long, action)]
    pub print: bool,

    /// Enable logging to 'gitu.log'
    #[clap(long, action)]
    pub log: bool,

    #[clap(long, action)]
    /// Print version
    pub version: bool,

    /// Config file to use
    #[clap(short, long)]
    pub config: Option<PathBuf>
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Show { reference: String },
}
