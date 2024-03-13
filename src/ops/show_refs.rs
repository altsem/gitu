use super::OpTrait;
use crate::{screen, state::State, term::Term, Res};
use std::rc::Rc;

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct ShowRefs;
impl OpTrait for ShowRefs {
    fn trigger(&self, state: &mut State, _term: &mut Term) -> Res<()> {
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
