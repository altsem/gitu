use super::OpTrait;
use crate::{state::State, Res};
use ratatui::{backend::Backend, prelude::Terminal};

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct ToggleSection;
impl<B: Backend> OpTrait<B> for ToggleSection {
    fn trigger(&self, state: &mut State, _term: &mut Terminal<B>) -> Res<()> {
        state.screen_mut().toggle_section();
        Ok(())
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct SelectPrevious;
impl<B: Backend> OpTrait<B> for SelectPrevious {
    fn trigger(&self, state: &mut State, _term: &mut Terminal<B>) -> Res<()> {
        state.screen_mut().select_previous();
        Ok(())
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct SelectNext;
impl<B: Backend> OpTrait<B> for SelectNext {
    fn trigger(&self, state: &mut State, _term: &mut Terminal<B>) -> Res<()> {
        state.screen_mut().select_next();
        Ok(())
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct HalfPageUp;
impl<B: Backend> OpTrait<B> for HalfPageUp {
    fn trigger(&self, state: &mut State, _term: &mut Terminal<B>) -> Res<()> {
        state.screen_mut().scroll_half_page_up();
        Ok(())
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct HalfPageDown;
impl<B: Backend> OpTrait<B> for HalfPageDown {
    fn trigger(&self, state: &mut State, _term: &mut Terminal<B>) -> Res<()> {
        state.screen_mut().scroll_half_page_down();
        Ok(())
    }
}
