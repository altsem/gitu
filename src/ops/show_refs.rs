use super::OpTrait;
use crate::{screen, state::State, Res};
use ratatui::{backend::Backend, prelude::Terminal};
use std::rc::Rc;

pub(crate) struct ShowRefs;
impl<B: Backend> OpTrait<B> for ShowRefs {
    fn trigger(&self, state: &mut State, _term: &mut Terminal<B>) -> Res<()> {
        goto_refs_screen(state);
        Ok(())
    }
}

fn goto_refs_screen(state: &mut State) {
    state.screens.drain(1..);
    let size = state.screens.last().unwrap().size;
    state.screens.push(
        screen::show_refs::create(Rc::clone(&state.config), Rc::clone(&state.repo), size)
            .expect("Couldn't create screen"),
    );
}
