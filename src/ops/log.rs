use super::{Action, OpTrait, TargetOpTrait};
use crate::{items::TargetData, screen, state::State, Res};
use ratatui::{backend::Backend, prelude::Terminal};
use std::rc::Rc;

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct LogCurrent;
impl<B: Backend> OpTrait<B> for LogCurrent {
    fn trigger(&self, state: &mut State, _term: &mut Terminal<B>) -> Res<()> {
        goto_log_screen(state, None);
        Ok(())
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct LogOther;
impl<B: Backend> TargetOpTrait<B> for LogOther {
    fn get_action(&self, target: TargetData) -> Option<Action<B>> {
        match target {
            TargetData::Commit(r) | TargetData::Branch(r) => Some(Box::new(move |state, _term| {
                goto_log_screen(state, Some(r.clone()));
                Ok(())
            })),
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
