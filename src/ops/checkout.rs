use super::{selected_rev, Action, OpTrait};
use crate::{
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
