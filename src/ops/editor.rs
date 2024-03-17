use super::{Action, OpTrait};
use crate::items::TargetData;
use derive_more::Display;
use std::rc::Rc;

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Refresh")]
pub(crate) struct Refresh;
impl OpTrait for Refresh {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|state, _term| state.screen_mut().update()))
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Toggle section")]
pub(crate) struct ToggleSection;
impl OpTrait for ToggleSection {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|state, _term| {
            state.screen_mut().toggle_section();
            Ok(())
        }))
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Select previous")]
pub(crate) struct SelectPrevious;
impl OpTrait for SelectPrevious {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|state, _term| {
            state.screen_mut().select_previous();
            Ok(())
        }))
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Select next")]
pub(crate) struct SelectNext;
impl OpTrait for SelectNext {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|state, _term| {
            state.screen_mut().select_next();
            Ok(())
        }))
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Half page up")]
pub(crate) struct HalfPageUp;
impl OpTrait for HalfPageUp {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|state, _term| {
            state.screen_mut().scroll_half_page_up();
            Ok(())
        }))
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Half page down")]
pub(crate) struct HalfPageDown;
impl OpTrait for HalfPageDown {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|state, _term| {
            state.screen_mut().scroll_half_page_down();
            Ok(())
        }))
    }
}
