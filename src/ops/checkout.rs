use super::{create_prompt_with_default, create_y_n_prompt, selected_rev, Action, OpTrait};
use crate::{
    error::Error, items::TargetData, menu::arg::Arg, prompt::PromptData, state::State, term::Term,
    Res,
};
use std::{process::Command, rc::Rc};
use tui_prompts::State as _;

pub(crate) fn init_args() -> Vec<Arg> {
    vec![]
}

pub(crate) struct Checkout;
impl OpTrait for Checkout {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(create_prompt_with_default(
            "Checkout",
            checkout,
            selected_rev,
            true,
        ))
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
        Some(Rc::new(|state: &mut State, _term: &mut Term| {
            state.close_menu();
            state.prompt.set(PromptData {
                prompt_text: "Create and checkout branch:".into(),
                update_fn: Rc::new(checkout_new_branch_prompt_update),
            });
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "Checkout new branch".into()
    }
}

fn checkout_new_branch_prompt_update(state: &mut State, term: &mut Term) -> Res<()> {
    if state.prompt.state.status().is_done() {
        let name = state.prompt.state.value().to_string();
        state.prompt.reset(term)?;

        let mut cmd = Command::new("git");
        cmd.args(["checkout", "-b", &name]);

        state.run_cmd(term, &[], cmd)?;
    }
    Ok(())
}

pub(crate) struct DeleteBranch;
impl OpTrait for DeleteBranch {
    fn get_action(&self, target: Option<&TargetData>) -> Option<Action> {
        match target {
            Some(TargetData::Branch(branch)) => {
                let branch = branch.clone();
                Some(create_y_n_prompt(
                    Rc::new(move |state, term| delete_branch(state, term, &branch)),
                    "Delete this branch?",
                ))
            }
            _ => Some(create_prompt_with_default(
                "Delete branch",
                delete_branch,
                selected_rev,
                true,
            )),
        }
    }

    fn display(&self, _state: &State) -> String {
        "Delete branch".into()
    }
}

fn delete_branch(state: &mut State, term: &mut Term, branch: &str) -> Res<()> {
    if branch.is_empty() {
        return Err(Error::InvalidBranch);
    }

    let delete_flag = {
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
        if crate::git::is_branch_unmerged(&state.repo, &target_branch).unwrap_or(false) {
            "-D"
        } else {
            "-d"
        }
    };

    let mut cmd = Command::new("git");
    cmd.args(["branch", delete_flag, branch]);
    state.run_cmd(term, &[], cmd)?;

    Ok(())
}
