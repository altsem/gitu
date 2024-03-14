use super::{Action, OpTrait, TargetOpTrait};
use crate::{items::TargetData, screen, state::State, term::Term, Res};
use derive_more::Display;
use std::rc::Rc;

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Log current")]
pub(crate) struct LogCurrent;
impl OpTrait for LogCurrent {
    fn trigger(&self, state: &mut State, _term: &mut Term) -> Res<()> {
        goto_log_screen(state, None);
        Ok(())
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Log other")]
pub(crate) struct LogOther;
impl TargetOpTrait for LogOther {
    fn get_action(&self, target: Option<&TargetData>) -> Option<Action> {
        match target.cloned() {
            Some(TargetData::Commit(r) | TargetData::Branch(r)) => {
                Some(Box::new(move |state, _term| {
                    goto_log_screen(state, Some(r.clone()));
                    Ok(())
                }))
            }
            _ => None,
        }
    }
}

fn goto_log_screen(state: &mut State, reference: Option<String>) {
    state.screens.drain(1..);
    let size = state.screens.last().unwrap().size;
    state.screens.push(
        screen::log::create(
            Rc::clone(&state.config),
            Rc::clone(&state.repo),
            size,
            reference,
        )
        .expect("Couldn't create screen"),
    );
}
