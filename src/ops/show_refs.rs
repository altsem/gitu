use super::{Action, OpTrait};
use crate::{items::TargetData, screen, state::State, term::Term};
use derive_more::Display;
use std::rc::Rc;

#[derive(Display)]
#[display(fmt = "Show Refs")]
pub(crate) struct ShowRefs;
impl OpTrait for ShowRefs {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|state: &mut State, _term: &mut Term| {
            goto_refs_screen(state);
            Ok(())
        }))
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
