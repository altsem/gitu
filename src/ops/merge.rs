use super::{Action, OpTrait, selected_rev};
use crate::{
    Res,
    app::{App, State},
    error::Error,
    item_data::ItemData,
    menu::arg::Arg,
    picker::{PickerData, PickerItem, PickerState},
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
    fn get_action(&self, _target: &ItemData) -> Option<Action> {
        Some(Rc::new(|app: &mut App, term: &mut Term| {
            let default_rev = selected_rev(app);
            let mut items = Vec::new();
            let mut seen = std::collections::HashSet::new();

            // Add default value first if it exists
            if let Some(ref default) = default_rev {
                items.push(PickerItem::new(
                    default.clone(),
                    PickerData::Revision(default.clone()),
                ));
                seen.insert(default.clone());
            }

            // Get current branch name to exclude it
            let current_branch = app.state.repo.head().ok()
                .and_then(|head| head.shorthand().map(|s| s.to_string()));

            // Get all branches
            let branches = app.state.repo.branches(None).map_err(Error::ListGitReferences)?;
            for branch in branches {
                let (branch, _) = branch.map_err(Error::ListGitReferences)?;
                if let Some(name) = branch.name().map_err(Error::ListGitReferences)? {
                    let name = name.to_string();
                    // Skip current branch and already seen names
                    if Some(&name) != current_branch.as_ref() && !seen.contains(&name) {
                        items.push(PickerItem::new(name.clone(), PickerData::Revision(name.clone())));
                        seen.insert(name);
                    }
                }
            }

            // Get all tags
            let tag_names = app.state.repo.tag_names(None).map_err(Error::ListGitReferences)?;
            for tag_name in tag_names.iter().flatten() {
                let tag_name = tag_name.to_string();
                if !seen.contains(&tag_name) {
                    items.push(PickerItem::new(tag_name.clone(), PickerData::Revision(tag_name.clone())));
                    seen.insert(tag_name);
                }
            }

            // Get all remote branches
            let references = app.state.repo.references().map_err(Error::ListGitReferences)?;
            for reference in references {
                let reference = reference.map_err(Error::ListGitReferences)?;
                if reference.is_remote() && let Some(name) = reference.shorthand() {
                    let name = name.to_string();
                    if !seen.contains(&name) {
                        items.push(PickerItem::new(name.clone(), PickerData::Revision(name.clone())));
                        seen.insert(name);
                    }
                }
            }

            // Allow custom input to support commit hashes, relative refs (e.g., HEAD~3),
            // and other git revisions not in the predefined list
            let picker = PickerState::new("Merge", items, true);
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
