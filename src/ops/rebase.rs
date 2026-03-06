use super::{Action, OpTrait};
use crate::{
    Res,
    app::{App, State},
    git,
    item_data::{ItemData, Ref},
    menu::arg::Arg,
    picker::{PickerParams, PickerState},
    term::Term,
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
    fn get_action(&self, _target: &ItemData) -> Option<Action> {
        Some(Rc::new(|app: &mut App, term: &mut Term| {
            let mut cmd = Command::new("git");
            cmd.args(["rebase", "--continue"]);

            app.close_menu();
            app.run_cmd_interactive(term, cmd)?;
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "continue".into()
    }
}

pub(crate) struct RebaseAbort;
impl OpTrait for RebaseAbort {
    fn get_action(&self, _target: &ItemData) -> Option<Action> {
        Some(Rc::new(|app: &mut App, term: &mut Term| {
            let mut cmd = Command::new("git");
            cmd.args(["rebase", "--abort"]);

            app.close_menu();
            app.run_cmd(term, &[], cmd)?;
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "abort".into()
    }
}

pub(crate) struct RebaseElsewhere;
impl OpTrait for RebaseElsewhere {
    fn get_action(&self, target: &ItemData) -> Option<Action> {
        let default_ref = if let ItemData::Reference { kind, .. } = target {
            Some(kind.clone())
        } else {
            None
        };

        Some(Rc::new(move |app: &mut App, term: &mut Term| {
            let args = app.state.pending_menu.as_ref().unwrap().args();
            app.close_menu();
            let result = app.pick(
                term,
                PickerState::with_refs(PickerParams {
                    prompt: "Rebase onto".into(),
                    refs: &git::branches_tags(&app.state.repo)?,
                    exclude_ref: git::head_ref(&app.state.repo)?,
                    default: default_ref.clone().map(crate::item_data::Rev::Ref),
                    allow_custom_input: true,
                }),
            )?;

            if let Some(data) = result {
                rebase_elsewhere(app, term, data.display(), &args)?;
            }
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "onto elsewhere".into()
    }
}

fn rebase_elsewhere(
    app: &mut App,
    term: &mut Term,
    rev: &str,
    args: &[std::ffi::OsString],
) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.arg("rebase");
    cmd.args(args);
    cmd.arg(rev);

    app.close_menu();
    app.run_cmd_interactive(term, cmd)?;
    Ok(())
}

pub(crate) struct RebaseInteractive;
impl OpTrait for RebaseInteractive {
    fn get_action(&self, target: &ItemData) -> Option<Action> {
        let action = match target {
            ItemData::Commit { oid, .. }
            | ItemData::Reference {
                kind: Ref::Tag(oid),
                ..
            }
            | ItemData::Reference {
                kind: Ref::Head(oid),
                ..
            } => {
                let rev = OsString::from(oid);
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

    fn display(&self, _state: &State) -> String {
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
    fn get_action(&self, target: &ItemData) -> Option<Action> {
        let action = match target {
            ItemData::Commit { oid, .. }
            | ItemData::Reference {
                kind: Ref::Tag(oid),
                ..
            }
            | ItemData::Reference {
                kind: Ref::Head(oid),
                ..
            } => {
                let rev = OsString::from(oid);
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

    fn display(&self, _state: &State) -> String {
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
