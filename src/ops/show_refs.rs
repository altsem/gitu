use super::{Action, OpTrait};
use crate::{
    app::{App, State},
    item_data::ItemData,
    screen,
    term::Term,
};
use std::{rc::Rc, sync::Arc};

pub(crate) struct ShowRefs;
impl OpTrait for ShowRefs {
    fn get_action(&self, _target: &ItemData) -> Option<Action> {
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
    app.state.return_to_main_screen();

    let size = {
        let this = &mut *app;
        this.state.get_focused_screen()
    }
    .size;
    app.close_menu();
    app.state.push_screen(
        screen::show_refs::create(
            Arc::clone(&app.state.config),
            Rc::clone(&app.state.repo),
            size,
        )
        .expect("Couldn't create screen"),
    );
}
