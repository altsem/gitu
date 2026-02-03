use super::{Action, OpTrait};
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
    picker::{BranchesAndTagsOptions, PickerState},
    term::Term,
};
use std::{process::Command, rc::Rc};

pub(crate) fn init_args() -> Vec<Arg> {
    vec![]
}

pub(crate) struct Checkout;
impl OpTrait for Checkout {
    fn get_action(&self, target: &ItemData) -> Option<Action> {
        let default_ref = target.to_ref_kind();

        Some(Rc::new(move |app: &mut App, term: &mut Term| {
            // Allow custom input to support checking out other revisions not in the list
            let picker = PickerState::for_branches_and_tags(
                "Checkout",
                &app.state.repo,
                BranchesAndTagsOptions {
                    exclude_head: true,
                    allow_custom_input: true,
                    default: default_ref.clone(),
                },
            )?;
            match app.picker(term, picker)? {
                Some(data) => checkout(app, term, data.display()),
                None => Ok(()), // picker got cancelled
            }
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
            let start_point_picker = PickerState::for_branches_and_tags(
                "Create branch starting at",
                &app.state.repo,
                BranchesAndTagsOptions {
                    exclude_head: false,
                    allow_custom_input: true,
                    default: {
                        let head = app.state.repo.head().map_err(Error::GetHead)?;
                        RefKind::from_reference(&head)
                    },
                },
            )?;

            let Some(starting_point) = app.picker(term, start_point_picker)? else {
                return Ok(());
            };

            let branch_name = app.prompt(
                term,
                &PromptParams {
                    prompt: "Create and checkout branch",
                    ..Default::default()
                },
            )?;

            checkout_new_branch_prompt_update(app, term, &branch_name, starting_point.display())?;
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "Checkout new branch".into()
    }
}

fn checkout_new_branch_prompt_update(
    app: &mut App,
    term: &mut Term,
    branch_name: &str,
    starting_point: &str,
) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.args(["checkout", "-b", branch_name, starting_point]);

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

            let branch_name = app.prompt(
                term,
                &PromptParams {
                    prompt: "Delete",
                    create_default_value: Box::new(move |_| default.clone()),
                    ..Default::default()
                },
            )?;

            delete(app, term, &branch_name)?;
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
