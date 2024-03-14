use super::OpTrait;
use crate::{items::TargetData, prompt::PromptData, state::State, term::Term, Res};
use derive_more::Display;
use std::process::Command;
use tui_prompts::State as _;

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Checkout branch/revision")]
pub(crate) struct Checkout;
impl OpTrait for Checkout {
    fn trigger(&self, state: &mut State, _term: &mut Term) -> Res<()> {
        let prompt_text = if let Some(branch_or_revision) = default_branch_or_revision(state) {
            format!("Checkout (default {}):", branch_or_revision).into()
        } else {
            "Checkout:".into()
        };

        state.prompt.set(PromptData {
            prompt_text,
            update_fn: Box::new(checkout_prompt_update),
        });
        Ok(())
    }
}

fn checkout_prompt_update(state: &mut State, term: &mut Term) -> Res<()> {
    if state.prompt.state.status().is_done() {
        let input = state.prompt.state.value().to_string();
        let branch_or_revision = match (input.as_str(), default_branch_or_revision(state)) {
            ("", None) => "",
            ("", Some(default)) => default,
            (value, _) => value,
        };

        let mut cmd = Command::new("git");
        cmd.args(["checkout", &branch_or_revision]);

        state.run_external_cmd(term, &[], cmd)?;
        state.prompt.reset(term)?;
    }
    Ok(())
}

fn default_branch_or_revision(state: &State) -> Option<&str> {
    match &state.screen().get_selected_item().target_data {
        Some(TargetData::Branch(branch)) => Some(branch),
        Some(TargetData::Commit(commit)) => Some(commit),
        _ => None,
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Checkout new branch")]
pub(crate) struct CheckoutNewBranch;
impl OpTrait for CheckoutNewBranch {
    fn trigger(&self, state: &mut State, _term: &mut Term) -> Res<()> {
        state.prompt.set(PromptData {
            prompt_text: "Create and checkout branch:".into(),
            update_fn: Box::new(checkout_new_branch_prompt_update),
        });
        Ok(())
    }
}

fn checkout_new_branch_prompt_update(state: &mut State, term: &mut Term) -> Res<()> {
    if state.prompt.state.status().is_done() {
        let name = state.prompt.state.value().to_string();
        let mut cmd = Command::new("git");
        cmd.args(["checkout", "-b", &name]);

        state.run_external_cmd(term, &[], cmd)?;
        state.prompt.reset(term)?;
    }
    Ok(())
}
