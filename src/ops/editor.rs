use super::{Action, OpTrait};
use crate::{items::TargetData, menu::PendingMenu, screen::NavMode, state::State, term::Term};
use derive_more::Display;
use std::rc::Rc;

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Quit/Close")]
pub(crate) struct Quit;
impl OpTrait for Quit {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|state, term| {
            match state.pending_menu {
                None => {
                    if state.screens.len() == 1 {
                        let quit = Rc::new(|state: &mut State, _term: &mut Term| {
                            state.quit = true;
                            Ok(())
                        });

                        let mut action = if state.config.general.confirm_quit.enabled {
                            super::create_y_n_prompt(quit, "Really quit?")
                        } else {
                            quit
                        };

                        Rc::get_mut(&mut action).unwrap()(state, term)?;
                    } else {
                        state.screens.pop();
                        if let Some(screen) = state.screens.last_mut() {
                            screen.update()?;
                        }
                    }
                }
                _ => {
                    state.pending_menu = None;
                    return Ok(());
                }
            }

            Ok(())
        }))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Submenu")]
pub(crate) struct Menu(pub crate::menu::Menu);
impl OpTrait for Menu {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        let submenu = self.0;
        Some(Rc::new(move |state, _term| {
            state.pending_menu = Some(PendingMenu::init(submenu));
            Ok(())
        }))
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Refresh")]
pub(crate) struct Refresh;
impl OpTrait for Refresh {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|state, _term| state.screen_mut().update()))
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = _.0)]
pub(crate) struct ToggleArg(pub &'static str);
impl OpTrait for ToggleArg {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|state, _term| {
            if let Some(menu) = &mut state.pending_menu {
                menu.args
                    .entry(self.0.into())
                    .and_modify(|value| *value = !*value);
            }
            Ok(())
        }))
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
#[display(fmt = "Move up")]
pub(crate) struct MoveUp;
impl OpTrait for MoveUp {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|state, _term| {
            state.screen_mut().select_previous(NavMode::Normal);
            Ok(())
        }))
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Move down")]
pub(crate) struct MoveDown;
impl OpTrait for MoveDown {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|state, _term| {
            state.screen_mut().select_next(NavMode::Normal);
            Ok(())
        }))
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Move down line")]
pub(crate) struct MoveDownLine;
impl OpTrait for MoveDownLine {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|state, _term| {
            state.screen_mut().select_next(NavMode::IncludeHunkLines);
            Ok(())
        }))
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Move up line")]
pub(crate) struct MoveUpLine;
impl OpTrait for MoveUpLine {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|state, _term| {
            state
                .screen_mut()
                .select_previous(NavMode::IncludeHunkLines);
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
