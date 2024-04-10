use super::{create_rev_prompt, Action, OpTrait};
use crate::{items::TargetData, menu::arg::Arg, prompt::PromptData, state::State, term::Term, Res};
use derive_more::Display;
use std::{ffi::OsString, process::Command, rc::Rc};
use tui_prompts::State as _;

pub(crate) const ARGS: &[Arg] = &[];

#[derive(Display)]
#[display(fmt = "Checkout branch/revision")]
pub(crate) struct Checkout;
impl OpTrait for Checkout {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(create_rev_prompt("Checkout", checkout))
    }
}

fn checkout(state: &mut State, term: &mut Term, args: &[OsString], rev: &str) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.args(["checkout"]);
    cmd.args(args);
    cmd.arg(rev);

    state.run_cmd(term, &[], cmd)?;
    Ok(())
}

#[derive(Display)]
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
        state.prompt.reset(term)?;

        let mut cmd = Command::new("git");
        cmd.args(["checkout", "-b", &name]);
        state.run_cmd(term, &[], cmd)?;
    }
    Ok(())
}
