use super::{Action, OpTrait};
use crate::{items::TargetData, screen, state::State, term::Term};
use derive_more::Display;
use std::rc::Rc;

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
    fn get_action(&self, target: Option<&TargetData>) -> Option<Action> {
        match target.cloned() {
            Some(TargetData::Commit(r) | TargetData::Branch(r)) => {
                Some(Rc::new(move |state, _term| {
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
