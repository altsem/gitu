use super::{create_rev_prompt, Action, OpTrait};
use crate::{items::TargetData, prompt::PromptData, state::State, term::Term, Res};
use derive_more::Display;
use std::{process::Command, rc::Rc};
use tui_prompts::State as _;

pub(crate) fn args() -> &'static [(&'static str, bool)] {
    &[]
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Checkout branch/revision")]
pub(crate) struct Checkout;
impl OpTrait for Checkout {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(create_rev_prompt("Checkout", checkout))
    }
}

fn checkout(state: &mut State, term: &mut Term, result: &str) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.args(["checkout", &result]);

    state.run_cmd(term, &[], cmd)?;
    Ok(())
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Checkout new branch")]
pub(crate) struct CheckoutNewBranch;
impl OpTrait for CheckoutNewBranch {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|state: &mut State, _term: &mut Term| {
            state.prompt.set(PromptData {
                prompt_text: "Create and checkout branch:".into(),
                update_fn: Rc::new(checkout_new_branch_prompt_update),
            });
            Ok(())
        }))
    }
}

fn checkout_new_branch_prompt_update(state: &mut State, term: &mut Term) -> Res<()> {
    if state.prompt.state.status().is_done() {
        let name = state.prompt.state.value().to_string();
        let mut cmd = Command::new("git");
        cmd.args(["checkout", "-b", &name]);

        state.run_cmd(term, &[], cmd)?;
        state.prompt.reset(term)?;
    }
    Ok(())
}
