use super::{Action, OpTrait};
use crate::{
    app::{App, State},
    items::TargetData,
    screen,
    term::Term,
};
use std::rc::Rc;

pub(crate) struct ShowRefs;
impl OpTrait for ShowRefs {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|app: &mut App, _term: &mut Term| {
            goto_refs_screen(app);
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "Show Refs".into()
    }
}

fn goto_refs_screen(app: &mut App) {
    app.state.screens.drain(1..);
    let size = app.state.screens.last().unwrap().size;
    app.close_menu();
    app.state.screens.push(
        screen::show_refs::create(
            Rc::clone(&app.state.config),
            Rc::clone(&app.state.repo),
            size,
        )
        .expect("Couldn't create screen"),
    );
}
