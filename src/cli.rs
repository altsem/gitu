use std::io;
use std::path::PathBuf;

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;

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
    pub config: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Show {
        reference: String,
    },
    Blame {
        file: String,
        #[clap(short, long)]
        rev: Option<String>,
    },
    /// Print a shell completion script to stdout.
    ///
    /// Example (bash): `gitu completion bash > ~/.local/share/bash-completion/completions/gitu`
    Completion {
        /// The shell to generate a completion script for
        shell: Shell,
    },
}

/// Write a shell completion script for `gitu` to the given writer.
pub fn completions(shell: Shell, out: &mut impl io::Write) {
    let mut cmd = Args::command();
    let bin_name = cmd.get_name().to_string();
    clap_complete::generate(shell, &mut cmd, bin_name, out);
}
