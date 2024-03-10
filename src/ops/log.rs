use super::OpTrait;
use crate::{screen, state::State, Res};
use ratatui::{backend::Backend, prelude::Terminal};
use std::rc::Rc;

pub(crate) struct LogCurrent;
impl<B: Backend> OpTrait<B> for LogCurrent {
    fn trigger(&self, state: &mut State, _term: &mut Terminal<B>) -> Res<()> {
        goto_log_screen(state, None);
        Ok(())
    }
}

pub(crate) fn goto_log_screen(state: &mut State, reference: Option<String>) {
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
