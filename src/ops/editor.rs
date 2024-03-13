use super::OpTrait;
use crate::{state::State, term::Term, Res};

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct ToggleSection;
impl OpTrait for ToggleSection {
    fn trigger(&self, state: &mut State, _term: &mut Term) -> Res<()> {
        state.screen_mut().toggle_section();
        Ok(())
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct SelectPrevious;
impl OpTrait for SelectPrevious {
    fn trigger(&self, state: &mut State, _term: &mut Term) -> Res<()> {
        state.screen_mut().select_previous();
        Ok(())
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct SelectNext;
impl OpTrait for SelectNext {
    fn trigger(&self, state: &mut State, _term: &mut Term) -> Res<()> {
        state.screen_mut().select_next();
        Ok(())
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct HalfPageUp;
impl OpTrait for HalfPageUp {
    fn trigger(&self, state: &mut State, _term: &mut Term) -> Res<()> {
        state.screen_mut().scroll_half_page_up();
        Ok(())
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct HalfPageDown;
impl OpTrait for HalfPageDown {
    fn trigger(&self, state: &mut State, _term: &mut Term) -> Res<()> {
        state.screen_mut().scroll_half_page_down();
        Ok(())
    }
}
