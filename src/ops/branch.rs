use super::{Action, OpTrait, selected_rev};
use crate::{
    Res,
    app::{App, PromptParams, State},
    error::Error,
    git::{
        does_branch_exist, get_current_branch, get_current_branch_name, is_branch_merged,
        remote::get_branch_upstream,
    },
    item_data::{ItemData, RefKind},
    menu::arg::Arg,
    picker::{PickerData, PickerItem, PickerState},
    term::Term,
};
use std::{process::Command, rc::Rc};

pub(crate) fn init_args() -> Vec<Arg> {
    vec![]
}

pub(crate) struct Checkout;
impl OpTrait for Checkout {
    fn get_action(&self, _target: &ItemData) -> Option<Action> {
        Some(Rc::new(move |app: &mut App, term: &mut Term| {
            let picker = create_branch_picker(app, "Checkout", true)?;
            let result = app.picker(term, picker)?;

            if let Some(data) = result {
                let rev = data.display();
                checkout(app, term, rev)?;
            }

            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "Checkout branch/revision".into()
    }
}

fn checkout(app: &mut App, term: &mut Term, rev: &str) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.args(["checkout", rev]);

    app.close_menu();
    app.run_cmd(term, &[], cmd)?;
    Ok(())
}

pub(crate) struct CheckoutNewBranch;
impl OpTrait for CheckoutNewBranch {
    fn get_action(&self, _target: &ItemData) -> Option<Action> {
        Some(Rc::new(|app: &mut App, term: &mut Term| {
            let branch_name = app.prompt(
                term,
                &PromptParams {
                    prompt: "Create and checkout branch:",
                    ..Default::default()
                },
            )?;

            checkout_new_branch_prompt_update(app, term, &branch_name)?;
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "Checkout new branch".into()
    }
}

fn checkout_new_branch_prompt_update(app: &mut App, term: &mut Term, branch_name: &str) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.args(["checkout", "-b", branch_name]);

    app.close_menu();
    app.run_cmd(term, &[], cmd)?;
    Ok(())
}

pub(crate) struct Delete;
impl OpTrait for Delete {
    fn get_action(&self, target: &ItemData) -> Option<Action> {
        let default = match target {
            ItemData::Reference {
                kind: RefKind::Branch(branch),
                ..
            } => Some(branch.clone()),
            _ => None,
        };

        Some(Rc::new(move |app: &mut App, term: &mut Term| {
            let default = default.clone();
            let picker = create_branch_picker_with_default(app, "Delete", true, default)?;
            let result = app.picker(term, picker)?;

            if let Some(data) = result {
                let branch_name = data.display();
                delete(app, term, branch_name)?;
            }

            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "Delete branch".into()
    }
}

pub fn delete(app: &mut App, term: &mut Term, branch_name: &str) -> Res<()> {
    if branch_name.is_empty() {
        return Err(Error::BranchNameRequired);
    }

    if get_current_branch_name(&app.state.repo).unwrap() == branch_name {
        return Err(Error::CannotDeleteCurrentBranch);
    }

    let mut cmd = Command::new("git");
    cmd.args(["branch", "-d"]);

    if !is_branch_merged(&app.state.repo, branch_name).unwrap_or(false) {
        app.confirm(term, "Branch is not fully merged. Really delete? (y or n)")?;
        cmd.arg("-f");
    }

    cmd.arg(branch_name);

    app.close_menu();
    app.run_cmd(term, &[], cmd)?;
    Ok(())
}

pub(crate) struct Spinoff;
impl OpTrait for Spinoff {
    fn get_action(&self, target: &ItemData) -> Option<Action> {
        let default = match target {
            ItemData::Reference {
                kind: RefKind::Branch(branch),
                ..
            } => Some(branch.clone()),
            _ => None,
        };

        Some(Rc::new(move |app: &mut App, term: &mut Term| {
            let default = default.clone();

            let new_branch_name = app.prompt(
                term,
                &PromptParams {
                    prompt: "Name for new branch",
                    create_default_value: Box::new(move |_| default.clone()),
                    ..Default::default()
                },
            )?;

            if new_branch_name.is_empty() {
                return Err(Error::BranchNameRequired);
            }

            if does_branch_exist(&app.state.repo, &new_branch_name)? {
                return Err(Error::SpinoffBranchExists(new_branch_name.to_string()));
            }

            let current_branch = get_current_branch(&app.state.repo)?;
            let current_branch_name = get_current_branch_name(&app.state.repo)?;

            if current_branch_name == new_branch_name {
                return Err(Error::CannotSpinoffCurrentBranch);
            }

            let base_commit_oid = app.state.repo.head().map_err(Error::GetHead)?.target();

            let upstream_branch_commit_oid = get_branch_upstream(&current_branch)?
                .map(|branch| branch.into_reference())
                .map(|x| x.target());

            drop(current_branch);

            app.close_menu();

            // Checkout new branch
            let mut cmd = Command::new("git");
            cmd.args(["checkout", "-b", &new_branch_name]);
            app.run_cmd(term, &[], cmd)?;

            let Some(upstream_branch_commit_oid) = upstream_branch_commit_oid else {
                app.display_info(format!("Branch {current_branch_name} not changed"));
                return Ok(());
            };

            if base_commit_oid == upstream_branch_commit_oid {
                app.display_info(format!("Branch {current_branch_name} not changed"));
                return Ok(());
            }

            let base_oid = base_commit_oid.ok_or(Error::BaseCommitOid)?;
            let upstream_oid = upstream_branch_commit_oid.ok_or(Error::UpstreamCommitOid)?;
            let merge_base = &app.state.repo.merge_base(base_oid, upstream_oid).unwrap();

            let mut cmd = Command::new("git");
            cmd.args([
                "update-ref",
                "-m",
                &format!(r##""reset: moving to {merge_base}""##),
                &format!("refs/heads/{current_branch_name}"),
                &merge_base.to_string(),
            ]);
            app.run_cmd(term, &[], cmd)?;

            app.display_info(format!(
                "Branch {current_branch_name} was reset to {merge_base}"
            ));

            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "Spinoff branch".into()
    }
}

fn create_branch_picker(
    app: &App,
    prompt: &'static str,
    exclude_current: bool,
) -> Result<PickerState, Error> {
    create_branch_picker_with_default(app, prompt, exclude_current, selected_rev(app))
}

fn create_branch_picker_with_default(
    app: &App,
    prompt: &'static str,
    exclude_current: bool,
    default_rev: Option<String>,
) -> Result<PickerState, Error> {
    let mut items = Vec::new();
    let mut seen = std::collections::HashSet::new();

    // Get current branch name if we need to exclude it
    let current_branch = if exclude_current {
        app.state
            .repo
            .head()
            .ok()
            .and_then(|head| head.shorthand().map(|s| s.to_string()))
    } else {
        None
    };

    // Add default value first if it exists and is not current branch
    if let Some(ref default) = default_rev
        && Some(default.as_str()) != current_branch.as_deref()
    {
        items.push(PickerItem::new(
            default.clone(),
            PickerData::Revision(default.clone()),
        ));
        seen.insert(default.clone());
    }

    // Get all branches (exclude current if needed)
    let branches = app
        .state
        .repo
        .branches(None)
        .map_err(Error::ListGitReferences)?;
    for branch in branches {
        let (branch, _) = branch.map_err(Error::ListGitReferences)?;
        if let Some(name) = branch.name().map_err(Error::ListGitReferences)? {
            let name = name.to_string();
            // Skip current branch and already seen names
            if Some(&name) != current_branch.as_ref() && !seen.contains(&name) {
                items.push(PickerItem::new(
                    name.clone(),
                    PickerData::Revision(name.clone()),
                ));
                seen.insert(name);
            }
        }
    }

    // Get all tags
    let tag_names = app
        .state
        .repo
        .tag_names(None)
        .map_err(Error::ListGitReferences)?;
    for tag_name in tag_names.iter().flatten() {
        let tag_name = tag_name.to_string();
        if !seen.contains(&tag_name) {
            items.push(PickerItem::new(
                tag_name.clone(),
                PickerData::Revision(tag_name.clone()),
            ));
            seen.insert(tag_name);
        }
    }

    // Get all remote branches
    let references = app
        .state
        .repo
        .references()
        .map_err(Error::ListGitReferences)?;
    for reference in references {
        let reference = reference.map_err(Error::ListGitReferences)?;
        if reference.is_remote()
            && let Some(name) = reference.shorthand()
        {
            let name = name.to_string();
            if !seen.contains(&name) {
                items.push(PickerItem::new(
                    name.clone(),
                    PickerData::Revision(name.clone()),
                ));
                seen.insert(name);
            }
        }
    }

    // Allow custom input to support commit hashes, relative refs (e.g., HEAD~3),
    // and other git revisions not in the predefined list
    Ok(PickerState::new(prompt, items, true))
}
