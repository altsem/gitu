use super::{Action, OpTrait};
use crate::item_data::RefKind;
use crate::{
    Res,
    app::{App, State},
    error::Error,
    item_data::ItemData,
    menu::arg::Arg,
    picker::PickerState,
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
        let default_ref = if let ItemData::Reference { kind, .. } = target {
            Some(kind.clone())
        } else {
            None
        };

        Some(Rc::new(move |app: &mut App, term: &mut Term| {
            // Get current HEAD reference to exclude it from picker
            let exclude_ref = {
                let head = app.state.repo.head().map_err(Error::GetHead)?;
                RefKind::from_reference(&head)
            };

            // Collect all branches (local and remote)
            let branches = app
                .state
                .repo
                .branches(None)
                .map_err(Error::ListGitReferences)?
                .filter_map(|branch| {
                    let (branch, _) = branch.ok()?;
                    RefKind::from_reference(branch.get())
                });

            // Collect all tags
            let tags: Vec<RefKind> = app
                .state
                .repo
                .tag_names(None)
                .map_err(Error::ListGitReferences)?
                .into_iter()
                .flatten()
                .map(|tag_name| RefKind::Tag(tag_name.to_string()))
                .collect();

            let all_refs: Vec<RefKind> = branches.chain(tags).collect();

            // Allow custom input to support commit hashes, relative refs (e.g., HEAD~3),
            // and other git revisions not in the predefined list
            let picker =
                PickerState::with_refs("Merge", all_refs, exclude_ref, default_ref.clone(), true);
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
