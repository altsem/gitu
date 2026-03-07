use super::{Action, OpTrait};
use crate::git;
use crate::picker::PickerParams;
use crate::{
    Res,
    app::{App, State},
    item_data::ItemData,
    menu::arg::Arg,
    picker::PickerState,
    term::Term,
};

use std::ffi::OsString;
use std::{process::Command, rc::Rc};

pub(crate) fn init_args() -> Vec<Arg> {
    vec![
        Arg::new_flag("--ff-only", "Fast-forward only", false),
        Arg::new_flag("--no-ff", "No fast-forward", false),
    ]
}

pub(crate) struct MergeContinue;
impl OpTrait for MergeContinue {
    fn get_action(&self, _target: &ItemData) -> Option<Action> {
        Some(Rc::new(|app: &mut App, term: &mut Term| {
            let mut cmd = Command::new("git");
            cmd.args(["merge", "--continue"]);

            app.close_menu();
            app.run_cmd_interactive(term, cmd)?;
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "continue".into()
    }
}

pub(crate) struct MergeAbort;
impl OpTrait for MergeAbort {
    fn get_action(&self, _target: &ItemData) -> Option<Action> {
        Some(Rc::new(|app: &mut App, term: &mut Term| {
            let mut cmd = Command::new("git");
            cmd.args(["merge", "--abort"]);

            app.close_menu();
            app.run_cmd_interactive(term, cmd)?;
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "abort".into()
    }
}

fn merge(app: &mut App, term: &mut Term, rev: &str, args: &[OsString]) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.arg("merge");
    cmd.args(args);
    cmd.arg(rev);

    app.close_menu();
    app.run_cmd_interactive(term, cmd)?;
    Ok(())
}

pub(crate) struct Merge;
impl OpTrait for Merge {
    fn get_action(&self, target: &ItemData) -> Option<Action> {
        let rev = target.rev();
        Some(Rc::new(move |app: &mut App, term: &mut Term| {
            let args = app.state.pending_menu.as_ref().unwrap().args();
            app.close_menu();
            let result = app.pick(
                term,
                PickerState::with_refs(PickerParams {
                    prompt: "Merge".into(),
                    refs: &git::branches_tags(&app.state.repo)?,
                    exclude_ref: git::head_ref(&app.state.repo)?,
                    default: rev.clone(),
                    allow_custom_input: true,
                }),
            )?;

            if let Some(data) = result {
                let rev = data.display();
                merge(app, term, rev, &args)?;
            }

            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "merge".into()
    }
}
