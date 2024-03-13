use super::{Op, OpTrait};
use crate::{items::TargetData, state::State, term::Term, Res};
use std::{borrow::Cow, process::Command};
use tui_prompts::{prelude::Status, State as _};

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Checkout;
impl OpTrait for Checkout {
    fn trigger(&self, state: &mut State, _term: &mut Term) -> Res<()> {
        state.prompt.set(Op::Checkout);
        Ok(())
    }

    fn format_prompt(&self, state: &State) -> Cow<'static, str> {
        if let Some(branch_or_revision) = default_branch_or_revision(state) {
            format!("Checkout (default {}):", branch_or_revision).into()
        } else {
            "Checkout:".into()
        }
    }

    fn prompt_update(&self, status: Status, state: &mut State, term: &mut Term) -> Res<()> {
        if status.is_done() {
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
}

fn default_branch_or_revision(state: &State) -> Option<&str> {
    match &state.screen().get_selected_item().target_data {
        Some(TargetData::Branch(branch)) => Some(branch),
        Some(TargetData::Commit(commit)) => Some(commit),
        _ => None,
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct CheckoutNewBranch;
impl OpTrait for CheckoutNewBranch {
    fn trigger(&self, state: &mut State, _term: &mut Term) -> Res<()> {
        state.prompt.set(Op::CheckoutNewBranch);
        Ok(())
    }

    fn format_prompt(&self, _state: &State) -> Cow<'static, str> {
        "Create and checkout branch:".into()
    }

    fn prompt_update(&self, status: Status, state: &mut State, term: &mut Term) -> Res<()> {
        if status.is_done() {
            let name = state.prompt.state.value().to_string();
            let mut cmd = Command::new("git");
            cmd.args(["checkout", "-b", &name]);

            state.run_external_cmd(term, &[], cmd)?;
            state.prompt.reset(term)?;
        }
        Ok(())
    }
}
