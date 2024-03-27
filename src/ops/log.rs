use super::{Action, OpTrait};
use crate::{
    items::TargetData, prompt::PromptData, screen, state::State, term::Term, ErrorBuffer, Res,
};
use derive_more::Display;
use git2::Oid;
use std::rc::Rc;
use tui_prompts::State as _;

pub(crate) fn args() -> &'static [(&'static str, bool)] {
    &[]
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Log current")]
pub(crate) struct LogCurrent;
impl OpTrait for LogCurrent {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|state: &mut State, _term: &mut Term| {
            goto_log_screen(state, None);
            Ok(())
        }))
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Log other")]
pub(crate) struct LogOther;
impl OpTrait for LogOther {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(move |state, _term| {
            let prompt_text = if let Some(branch_or_revision) = default_branch_or_revision(state) {
                format!("Log rev (default {}):", branch_or_revision).into()
            } else {
                "Log rev:".into()
            };

            state.prompt.set(PromptData {
                prompt_text,
                update_fn: Rc::new(log_other_prompt_update),
            });
            Ok(())
        }))
    }
}

fn log_other_prompt_update(state: &mut State, term: &mut Term) -> Res<()> {
    if state.prompt.state.status().is_done() {
        let input = state.prompt.state.value().to_string();

        let branch_or_revision = match (input.as_str(), default_branch_or_revision(state)) {
            ("", None) => "",
            ("", Some(default)) => default,
            (value, _) => value,
        };

        let oid = match state.repo.revparse_single(branch_or_revision) {
            Ok(rev) => rev.id(),
            Err(err) => {
                state.error_buffer = Some(ErrorBuffer(format!("Failed due to: {:?}", err.code())));
                state.prompt.reset(term)?;
                return Ok(());
            }
        };

        goto_log_screen(state, Some(oid));
        state.prompt.reset(term)?;
    }
    Ok(())
}

fn default_branch_or_revision(state: &State) -> Option<&str> {
    match &state.screen().get_selected_item().target_data {
        Some(TargetData::Commit(rev) | TargetData::Branch(rev)) => Some(rev),
        _ => None,
    }
}

fn goto_log_screen(state: &mut State, rev: Option<Oid>) {
    state.screens.drain(1..);
    let size = state.screens.last().unwrap().size;
    state.screens.push(
        screen::log::create(Rc::clone(&state.config), Rc::clone(&state.repo), size, rev)
            .expect("Couldn't create screen"),
    );
}
