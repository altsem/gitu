use super::{Action, OpTrait};
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
    fn get_action(&self, target: &ItemData) -> Option<Action> {
        use crate::item_data::RefKind;

        // Extract default ref from target if it's a Reference
        let default_ref = if let ItemData::Reference { kind, .. } = target {
            Some(kind.clone())
        } else {
            None
        };

        Some(Rc::new(move |app: &mut App, term: &mut Term| {
            let mut items = Vec::new();
            let mut shorthand_count: std::collections::HashMap<String, usize> =
                std::collections::HashMap::new();
            let mut branches: Vec<RefKind> = Vec::new();
            let mut tags: Vec<RefKind> = Vec::new();
            let mut remotes: Vec<RefKind> = Vec::new();

            // Get current HEAD reference to exclude it from picker
            // HEAD can be a branch, tag, remote, or detached (pointing to a commit)
            let current_ref = {
                let head = app.state.repo.head().map_err(Error::GetHead)?;
                RefKind::from_reference(&head)
            };

            // Collect all branches (local and remote)
            let branches_iter = app
                .state
                .repo
                .branches(None)
                .map_err(Error::ListGitReferences)?;
            for branch in branches_iter {
                let (branch, branch_type) = branch.map_err(Error::ListGitReferences)?;
                let branch_ref = branch.get();
                if let Some(ref_kind) = RefKind::from_reference(branch_ref) {
                    match branch_type {
                        git2::BranchType::Local => {
                            let name = ref_kind.shorthand().to_string();
                            *shorthand_count.entry(name).or_insert(0) += 1;
                            branches.push(ref_kind);
                        }
                        git2::BranchType::Remote => {
                            remotes.push(ref_kind);
                        }
                    }
                }
            }

            // Collect all tags (count for duplicate detection)
            let tag_names = app
                .state
                .repo
                .tag_names(None)
                .map_err(Error::ListGitReferences)?;
            for tag_name in tag_names.iter().flatten() {
                let tag_name = tag_name.to_string();
                let ref_kind = RefKind::Tag(tag_name.clone());
                *shorthand_count.entry(tag_name).or_insert(0) += 1;
                tags.push(ref_kind);
            }

            // Add default ref first if it exists
            if let Some(ref default) = default_ref {
                let shorthand = default.shorthand();
                let (display, refname) = match default {
                    RefKind::Remote(_) => {
                        // Remotes never have duplicates
                        (shorthand.to_string(), shorthand.to_string())
                    }
                    _ => {
                        let is_duplicate = shorthand_count.get(shorthand).is_some_and(|&c| c > 1);
                        if is_duplicate {
                            let full_refname = default.to_full_refname();
                            (full_refname.clone(), full_refname)
                        } else {
                            (shorthand.to_string(), shorthand.to_string())
                        }
                    }
                };
                items.push(PickerItem::new(display, PickerData::Revision(refname)));
            }

            // Add all refs (branches, then tags, then remotes)
            for ref_kind in branches.into_iter().chain(tags).chain(remotes) {
                let shorthand = ref_kind.shorthand();

                // Skip current ref (HEAD) - could be branch, tag, or remote
                if current_ref
                    .as_ref()
                    .is_some_and(|current| current.to_full_refname() == ref_kind.to_full_refname())
                {
                    continue;
                }

                // Skip if it's the same as default (compare by full refname to distinguish branch/tag)
                if default_ref
                    .as_ref()
                    .is_some_and(|d| d.to_full_refname() == ref_kind.to_full_refname())
                {
                    continue;
                }

                // Handle duplicates (only for branches and tags)
                let (display, refname) = match &ref_kind {
                    RefKind::Remote(_) => {
                        // Remotes never have duplicates (they have remote/ prefix)
                        (shorthand.to_string(), shorthand.to_string())
                    }
                    _ => {
                        let is_duplicate = shorthand_count.get(shorthand).is_some_and(|&c| c > 1);
                        if is_duplicate {
                            let full_refname = ref_kind.to_full_refname();
                            (full_refname.clone(), full_refname)
                        } else {
                            (shorthand.to_string(), shorthand.to_string())
                        }
                    }
                };
                items.push(PickerItem::new(display, PickerData::Revision(refname)));
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
