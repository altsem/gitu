use super::{Action, OpTrait};
use crate::{items::TargetData, screen, state::State, term::Term};
use std::rc::Rc;

pub(crate) struct ShowRefs;
impl OpTrait for ShowRefs {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|state: &mut State, _term: &mut Term| {
            goto_refs_screen(state);
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "Show Refs".into()
    }
}

fn goto_refs_screen(state: &mut State) {
    state.screens.drain(1..);
    let size = state.screens.last().unwrap().size;
    state.close_menu();
    state.screens.push(
        screen::show_refs::create(Rc::clone(&state.config), Rc::clone(&state.repo), size)
            .expect("Couldn't create screen"),
    );
}
