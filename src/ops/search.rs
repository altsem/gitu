use super::{Action, OpTrait};
use crate::{
    Res,
    app::{App, PromptParams, State},
    error::Error,
    item_data::ItemData,
    screen::SearchDirection,
    term::Term,
};
use std::rc::Rc;

pub(crate) struct Search;
impl OpTrait for Search {
    fn get_action(&self, _target: &ItemData) -> Option<Action> {
        Some(Rc::new(|app, term| {
            prompt_search(app, term, SearchDirection::Forward)
        }))
    }

    fn display(&self, _state: &State) -> String {
        "Search".into()
    }
}

pub(crate) struct SearchBackward;
impl OpTrait for SearchBackward {
    fn get_action(&self, _target: &ItemData) -> Option<Action> {
        Some(Rc::new(|app, term| {
            prompt_search(app, term, SearchDirection::Backward)
        }))
    }

    fn display(&self, _state: &State) -> String {
        "Search backward".into()
    }
}

pub(crate) struct SearchNext;
impl OpTrait for SearchNext {
    fn get_action(&self, _target: &ItemData) -> Option<Action> {
        Some(Rc::new(|app, _term| app.screen_mut().search_repeat(false)))
    }

    fn display(&self, _state: &State) -> String {
        "Next match".into()
    }
}

pub(crate) struct SearchPrevious;
impl OpTrait for SearchPrevious {
    fn get_action(&self, _target: &ItemData) -> Option<Action> {
        Some(Rc::new(|app, _term| app.screen_mut().search_repeat(true)))
    }

    fn display(&self, _state: &State) -> String {
        "Previous match".into()
    }
}

fn prompt_search(app: &mut App, term: &mut Term, direction: SearchDirection) -> Res<()> {
    let prompt = match direction {
        SearchDirection::Forward => "Search",
        SearchDirection::Backward => "Search backward",
    };

    let query = match app.prompt(
        term,
        &PromptParams {
            prompt,
            ..Default::default()
        },
    ) {
        Ok(query) if query.is_empty() => {
            app.screen_mut().clear_search();
            return Ok(());
        }
        Ok(query) => query,
        Err(Error::PromptAborted) => {
            app.screen_mut().clear_search();
            return Ok(());
        }
        Err(err) => return Err(err),
    };

    app.screen_mut().search(&query, direction)
}
