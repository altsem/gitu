use super::{Action, OpTrait, selected_rev};
use crate::{
    Res,
    app::{App, PromptParams, State},
    error::Error,
    git::{get_current_branch_name, is_branch_merged},
    item_data::{ItemData, RefKind},
    menu::arg::Arg,
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
            let rev = app.prompt(
                term,
                &PromptParams {
                    prompt: "Checkout",
                    create_default_value: Box::new(selected_rev),
                    ..Default::default()
                },
            )?;

            checkout(app, term, &rev)?;
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
