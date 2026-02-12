use super::{Action, OpTrait};
use crate::{
    Res,
    app::{App, State},
    item_data::ItemData,
    menu::arg::Arg,
    picker::{BranchesAndTagsOptions, PickerState},
    term::Term,
};

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

fn merge(app: &mut App, term: &mut Term, rev: &str) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.arg("merge");
    cmd.args(app.state.pending_menu.as_ref().unwrap().args());
    cmd.arg(rev);

    app.close_menu();
    app.run_cmd_interactive(term, cmd)?;
    Ok(())
}

pub(crate) struct Merge;
impl OpTrait for Merge {
    fn get_action(&self, target: &ItemData) -> Option<Action> {
        // Extract default ref from target if it's a Reference
        let default_ref = target.to_ref_kind();

        Some(Rc::new(move |app: &mut App, term: &mut Term| {
            // Allow custom input to support commit hashes, relative refs (e.g., HEAD~3),
            // and other git revisions not in the predefined list
            let picker = PickerState::for_branches_and_tags(
                "Merge",
                &app.state.repo,
                BranchesAndTagsOptions {
                    exclude_head: true,
                    allow_custom_input: true,
                    default: default_ref.clone(),
                },
            )?;
            let result = app.picker(term, picker)?;

            if let Some(data) = result {
                let rev = data.display();
                merge(app, term, rev)?;
            }

            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "merge".into()
    }
}
