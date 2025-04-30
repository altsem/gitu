use super::{selected_rev, Action, OpTrait};
use crate::{
    error::Error,
    items::TargetData,
    menu::arg::Arg,
    state::{PromptParams, State},
    term::Term,
    Res,
};
use std::{process::Command, rc::Rc};

pub(crate) fn init_args() -> Vec<Arg> {
    vec![]
}

pub(crate) struct Checkout;
impl OpTrait for Checkout {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(move |state: &mut State, term: &mut Term| {
            let rev = state.prompt(
                term,
                &PromptParams {
                    prompt: "Checkout",
                    create_default_value: Box::new(selected_rev),
                    ..Default::default()
                },
            )?;

            checkout(state, term, &rev)?;
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "Checkout branch/revision".into()
    }
}

fn checkout(state: &mut State, term: &mut Term, rev: &str) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.args(["checkout"]);
    cmd.args(state.pending_menu.as_ref().unwrap().args());
    cmd.arg(rev);

    state.close_menu();
    state.run_cmd(term, &[], cmd)?;
    Ok(())
}

pub(crate) struct CheckoutNewBranch;
impl OpTrait for CheckoutNewBranch {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|state: &mut State, term: &mut Term| {
            let branch_name = state.prompt(
                term,
                &PromptParams {
                    prompt: "Create and checkout branch:",
                    ..Default::default()
                },
            )?;

            checkout_new_branch_prompt_update(state, term, &branch_name)?;
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "Checkout new branch".into()
    }
}

fn checkout_new_branch_prompt_update(
    state: &mut State,
    term: &mut Term,
    branch_name: &str,
) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.args(["checkout", "-b", branch_name]);

    state.close_menu();
    state.run_cmd(term, &[], cmd)?;
    Ok(())
}

pub(crate) struct DeleteBranch;
impl OpTrait for DeleteBranch {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(move |state: &mut State, term: &mut Term| {
            let rev = state.prompt(
                term,
                &PromptParams {
                    prompt: "Delete branch",
                    create_default_value: Box::new(selected_rev),
                    ..Default::default()
                },
            )?;

            delete_branch(state, term, &rev)?;
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "Delete branch".into()
    }
}

fn delete_branch(state: &mut State, term: &mut Term, branch: &str) -> Res<()> {
    if branch.is_empty() {
        return Err(Error::InvalidBranch);
    }

    let is_unmerged = {
        let current_branch = crate::git::get_current_branch(&state.repo)?;
        let current_branch_name = match current_branch.name() {
            Ok(Some(name)) => name,
            Ok(None) => return Err(Error::CantGetBranchName),
            Err(e) => return Err(Error::CantGetBranch(e)),
        };

        if branch == current_branch_name {
            return Err(Error::CantDeleteCurrentBranch);
        }

        let target_branch = crate::git::get_branch(&state.repo, branch)?;

        // Get if branch is unmerged
        crate::git::is_branch_unmerged(&state.repo, &target_branch).unwrap_or(false)
    };

    if is_unmerged {
        let branch_to_delete = branch.to_string();

        let action = Rc::new(move |state: &mut State, term: &mut Term| {
            perform_branch_deletion(state, term, &branch_to_delete)
        });

        let prompt = create_y_n_prompt(action, "Branch is unmerged. Really delete?");
        prompt(state, term)?;

        Ok(())
    } else {
        perform_branch_deletion(state, term, branch)
    }
}

fn perform_branch_deletion(state: &mut State, term: &mut Term, branch: &str) -> Res<()> {
    let mut cmd = Command::new("git");

    cmd.args(["branch", "-D", branch]);
    state.run_cmd(term, &[], cmd)?;
    Ok(())
}
