use std::{rc::Rc, sync::Arc};

use crate::{
    app::State,
    error::Error,
    item_data::{ItemData, RefKind},
    screen,
};

use super::{Action, OpTrait};

pub(crate) struct Preview;
impl OpTrait for Preview {
    fn get_action(&self, target: &ItemData) -> Option<Action> {
        match target {
            ItemData::Commit { oid, .. }
            | ItemData::Reference {
                kind: RefKind::Tag(oid),
                ..
            }
            | ItemData::Reference {
                kind: RefKind::Branch(oid),
                ..
            } => goto_preview_screen(oid.clone()),
            ItemData::Stash { stash_ref, .. } => goto_stash_preview_screen(stash_ref.clone()),
            _ => None,
        }
    }

    fn is_target_op(&self) -> bool {
        true
    }

    fn display(&self, _: &State) -> String {
        "Preview".into()
    }
}

fn goto_preview_screen(oid: String) -> Option<Action> {
    Some(Rc::new(move |app, term| {
        app.state.set_preview_screen(
            screen::show::create(
                Arc::clone(&app.state.config),
                Rc::clone(&app.state.repo),
                term.size().map_err(Error::Term)?,
                oid.clone(),
            )
            .expect("Couldn't create screen"),
        );
        Ok(())
    }))
}

fn goto_stash_preview_screen(oid: String) -> Option<Action> {
    Some(Rc::new(move |app, term| {
        app.state.set_preview_screen(
            screen::show_stash::create(
                Arc::clone(&app.state.config),
                Rc::clone(&app.state.repo),
                term.size().map_err(Error::Term)?,
                oid.clone(),
            )
            .expect("Couldn't create screen"),
        );
        Ok(())
    }))
}
