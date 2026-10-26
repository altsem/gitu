use clap::Parser;
use clap_complete::Shell;

use crate::cli::{Args, Commands, completions};

#[test]
fn completion_subcommand_parses() {
    let args = Args::try_parse_from(["gitu", "completion", "fish"]).unwrap();
    assert!(matches!(
        args.command,
        Some(Commands::Completion { shell: Shell::Fish })
    ));
}

#[test]
fn completion_rejects_unknown_shell() {
    assert!(Args::try_parse_from(["gitu", "completion", "not-a-shell"]).is_err());
}

#[test]
fn generates_non_empty_script_for_every_shell() {
    for shell in [
        Shell::Bash,
        Shell::Elvish,
        Shell::Fish,
        Shell::PowerShell,
        Shell::Zsh,
    ] {
        let mut out = Vec::new();
        completions(shell, &mut out);
        let script = String::from_utf8(out).expect("completion script should be valid UTF-8");

        assert!(
            !script.trim().is_empty(),
            "{shell} completion script should not be empty"
        );
        assert!(
            script.contains("gitu"),
            "{shell} completion script should reference the binary name"
        );
    }
}
