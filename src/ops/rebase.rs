use super::{selected_rev, Action, OpTrait};
use crate::{
    app::{App, PromptParams},
    menu::arg::Arg,
    target_data::{RefKind, TargetData},
    term::Term,
    Res,
};
use std::{
    ffi::{OsStr, OsString},
    process::Command,
    rc::Rc,
};

pub(crate) fn init_args() -> Vec<Arg> {
    vec![
        Arg::new_flag("--keep-empty", "Keep empty commits", false),
        Arg::new_flag("--preserve-merges", "Preserve merges", false),
        Arg::new_flag(
            "--committer-date-is-author-date",
            "Lie about committer date",
            false,
        ),
        Arg::new_flag("--autosquash", "Autosquash", false),
        Arg::new_flag("--autostash", "Autostash", true),
        Arg::new_flag("--interactive", "Interactive", false),
        Arg::new_flag("--no-verify", "Disable hooks", false),
    ]
}

pub(crate) struct RebaseContinue;
impl OpTrait for RebaseContinue {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|app: &mut App, term: &mut Term| {
            let mut cmd = Command::new("git");
            cmd.args(["rebase", "--continue"]);

            app.close_menu();
            app.run_cmd_interactive(term, cmd)?;
            Ok(())
        }))
    }

    fn display(&self, _app: &App) -> String {
        "continue".into()
    }
}

pub(crate) struct RebaseAbort;
impl OpTrait for RebaseAbort {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|app: &mut App, term: &mut Term| {
            let mut cmd = Command::new("git");
            cmd.args(["rebase", "--abort"]);

            app.close_menu();
            app.run_cmd(term, &[], cmd)?;
            Ok(())
        }))
    }

    fn display(&self, _app: &App) -> String {
        "abort".into()
    }
}

pub(crate) struct RebaseElsewhere;
impl OpTrait for RebaseElsewhere {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(move |app: &mut App, term: &mut Term| {
            let rev = app.prompt(
                term,
                &PromptParams {
                    prompt: "Rebase onto",
                    create_default_value: Box::new(selected_rev),
                    ..Default::default()
                },
            )?;

            rebase_elsewhere(app, term, &rev)?;
            Ok(())
        }))
    }

    fn display(&self, _app: &App) -> String {
        "onto elsewhere".into()
    }
}

fn rebase_elsewhere(app: &mut App, term: &mut Term, rev: &str) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.arg("rebase");
    cmd.args(app.state.pending_menu.as_ref().unwrap().args());
    cmd.arg(rev);

    app.close_menu();
    app.run_cmd_interactive(term, cmd)?;
    Ok(())
}

pub(crate) struct RebaseInteractive;
impl OpTrait for RebaseInteractive {
    fn get_action(&self, target: Option<&TargetData>) -> Option<Action> {
        let action = match target {
            Some(
                TargetData::Commit(r)
                | TargetData::Reference(RefKind::Tag(r))
                | TargetData::Reference(RefKind::Branch(r)),
            ) => {
                let rev = OsString::from(r);
                Rc::new(move |app: &mut App, term: &mut Term| {
                    let args = app.state.pending_menu.as_ref().unwrap().args();
                    app.close_menu();
                    app.run_cmd_interactive(term, rebase_interactive_cmd(&args, &rev))
                })
            }
            _ => return None,
        };

        Some(action)
    }
    fn is_target_op(&self) -> bool {
        true
    }

    fn display(&self, _app: &App) -> String {
        "interactively".into()
    }
}

fn rebase_interactive_cmd(args: &[OsString], rev: &OsStr) -> Command {
    let mut cmd = Command::new("git");
    cmd.args(["rebase", "-i"]);
    cmd.args(args);
    cmd.arg(parent(rev));
    cmd
}

fn parent(reference: &OsStr) -> OsString {
    let mut parent = reference.to_os_string();
    parent.push("^");
    parent
}

pub(crate) struct RebaseAutosquash;
impl OpTrait for RebaseAutosquash {
    fn get_action(&self, target: Option<&TargetData>) -> Option<Action> {
        let action = match target {
            Some(
                TargetData::Commit(r)
                | TargetData::Reference(RefKind::Tag(r))
                | TargetData::Reference(RefKind::Branch(r)),
            ) => {
                let rev = OsString::from(r);
                Rc::new(move |app: &mut App, term: &mut Term| {
                    let args = app.state.pending_menu.as_ref().unwrap().args();
                    app.close_menu();
                    app.run_cmd_interactive(term, rebase_autosquash_cmd(&args, &rev))
                })
            }
            _ => return None,
        };

        Some(action)
    }
    fn is_target_op(&self) -> bool {
        true
    }

    fn display(&self, _app: &App) -> String {
        "autosquash".into()
    }
}

fn rebase_autosquash_cmd(args: &[OsString], rev: &OsStr) -> Command {
    let mut cmd = Command::new("git");
    cmd.args(["rebase", "-i", "--autosquash", "--keep-empty"]);
    cmd.args(args);
    cmd.arg(rev);
    cmd
}
