use super::{Action, OpTrait};
use crate::{
    app::{App, State},
    item_data::ItemData,
    menu::arg::Arg,
    term::Term,
};
use std::{
    ffi::{OsStr, OsString},
    process::Command,
    rc::Rc,
};

pub(crate) fn init_args() -> Vec<Arg> {
    vec![
        Arg::new_flag("--all", "Stage all modified and deleted files", false),
        Arg::new_flag("--allow-empty", "Allow empty commit", false),
        Arg::new_flag("--verbose", "Show diff of changes to be committed", false),
        Arg::new_flag("--no-verify", "Disable hooks", false),
        Arg::new_flag(
            "--reset-author",
            "Claim authorship and reset author date",
            false,
        ),
        // TODO -A Override the author (--author=)
        Arg::new_flag("--signoff", "Add Signed-off-by line", false),
        // TODO -C Reuse commit message (--reuse-message=)
    ]
}

pub(crate) struct Commit;
impl OpTrait for Commit {
    fn get_action(&self, _target: &ItemData) -> Option<Action> {
        Some(Rc::new(|app: &mut App, term: &mut Term| {
            let mut cmd = Command::new("git");
            cmd.args(["commit"]);
            cmd.args(app.state.pending_menu.as_ref().unwrap().args());

            app.close_menu();
            app.run_cmd_interactive(term, cmd)?;
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "Commit".into()
    }
}

pub(crate) struct CommitAmend;
impl OpTrait for CommitAmend {
    fn get_action(&self, _target: &ItemData) -> Option<Action> {
        Some(Rc::new(|app: &mut App, term: &mut Term| {
            let mut cmd = Command::new("git");
            cmd.args(["commit", "--amend"]);
            cmd.args(app.state.pending_menu.as_ref().unwrap().args());

            app.close_menu();
            app.run_cmd_interactive(term, cmd)?;
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "amend".into()
    }
}

pub(crate) struct CommitExtend;
impl OpTrait for CommitExtend {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|app: &mut App, term: &mut Term| {
            let mut cmd = Command::new("git");
            cmd.args(["commit", "--amend", "--no-edit"]);
            cmd.args(app.state.pending_menu.as_ref().unwrap().args());

            app.close_menu();
            app.run_cmd_interactive(term, cmd)?;
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "extend".into()
    }
}

pub(crate) struct CommitFixup;
impl OpTrait for CommitFixup {
    fn get_action(&self, target: &ItemData) -> Option<Action> {
        match target {
            ItemData::Commit { oid, .. } => {
                let rev = OsString::from(oid);

                Some(Rc::new(move |app: &mut App, term: &mut Term| {
                    let args = app.state.pending_menu.as_ref().unwrap().args();

                    app.close_menu();
                    app.run_cmd_interactive(term, commit_fixup_cmd(&args, &rev))
                }))
            }
            _ => None,
        }
    }

    fn is_target_op(&self) -> bool {
        true
    }

    fn display(&self, _state: &State) -> String {
        "fixup".into()
    }
}

fn commit_fixup_cmd(args: &[OsString], rev: &OsStr) -> Command {
    let mut cmd = Command::new("git");
    cmd.args(["commit", "--fixup"]);
    cmd.arg(rev);
    cmd.args(args);
    cmd
}

pub(crate) struct CommitInstantFixup;
impl OpTrait for CommitInstantFixup {
    fn get_action(&self, target: &ItemData) -> Option<Action> {
        match target {
            ItemData::Commit { oid, .. } => {
                let rev = OsString::from(oid);

                Some(Rc::new(move |app: &mut App, term: &mut Term| {
                    let args = app.state.pending_menu.as_ref().unwrap().args();

                    app.close_menu();

                    app.run_cmd(term, &[], commit_fixup_cmd(&args, &rev))?;
                    app.run_cmd(term, &[], rebase_autosquash_cmd(&rev))
                }))
            }
            _ => None,
        }
    }

    fn is_target_op(&self) -> bool {
        true
    }

    fn display(&self, _state: &State) -> String {
        "instant fixup".into()
    }
}

fn rebase_autosquash_cmd(rev: &OsStr) -> Command {
    let mut cmd = Command::new("git");
    cmd.args([
        "rebase",
        "-i",
        "-q",
        "--autostash",
        "--keep-empty",
        "--autosquash",
    ]);
    cmd.arg(parent(rev));
    cmd.env("GIT_SEQUENCE_EDITOR", ":");
    cmd
}

fn parent(reference: &OsStr) -> OsString {
    let mut parent = reference.to_os_string();
    parent.push("^");
    parent
}
