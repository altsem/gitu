use super::{set_prompt, Action, OpTrait};
use crate::{
    items::TargetData,
    menu::PendingMenu,
    screen::NavMode,
    state::{root_menu, State},
    term::Term,
    Res,
};
use derive_more::Display;
use std::rc::Rc;

#[derive(Display)]
#[display(fmt = "Quit/Close")]
pub(crate) struct Quit;
impl OpTrait for Quit {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|state, term| {
            let menu = state
                .pending_menu
                .as_ref()
                .map(|pending_menu| pending_menu.menu);

            if menu == root_menu(&state.config) {
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
            } else {
                state.pending_menu = root_menu(&state.config).map(PendingMenu::init);
                return Ok(());
            }

            Ok(())
        }))
    }
}

#[derive(Display)]
#[display(fmt = "Submenu")]
pub(crate) struct OpenMenu(pub crate::menu::Menu);
impl OpTrait for OpenMenu {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        let submenu = self.0;
        Some(Rc::new(move |state, _term| {
            state.pending_menu = Some(PendingMenu::init(submenu));
            Ok(())
        }))
    }
}

#[derive(Display)]
#[display(fmt = "Refresh")]
pub(crate) struct Refresh;
impl OpTrait for Refresh {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|state, _term| state.screen_mut().update()))
    }
}

#[derive(Display)]
#[display(fmt = _.0)]
pub(crate) struct ToggleArg(pub String);
impl OpTrait for ToggleArg {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        let arg_name = self.0.clone();
        Some(Rc::new(move |state, _term| {
            let mut need_prompt = None;
            let mut default = None;

            let maybe_entry = if let Some(menu) = &mut state.pending_menu {
                Some(menu.args.entry(arg_name.clone().into()))
            } else {
                None
            };

            if let Some(entry) = maybe_entry {
                entry.and_modify(|arg| {
                    if arg.is_active() {
                        arg.unset();
                    } else {
                        if arg.expects_value() {
                            default = arg.default_as_string();
                            need_prompt = Some(arg.display);
                        }
                    }
                });
            }

            if let Some(display) = need_prompt {
                set_prompt(
                    state,
                    display,
                    parse_and_set_arg,
                    Box::new(move |_| default.clone()),
                    arg_name.clone(),
                );
            }

            Ok(())
        }))
    }
}

fn parse_and_set_arg(
    state: &mut State,
    _term: &mut Term,
    _args: &[std::ffi::OsString],
    value: &str,
    arg: &String,
) -> Res<()> {
    let key: &str = arg;
    if let Some(menu) = &mut state.pending_menu {
        if let Some(entry) = menu.args.get_mut(key) {
            return entry.set(value);
        }
    }

    Ok(())
}

#[derive(Display)]
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

#[derive(Display)]
#[display(fmt = "Up")]
pub(crate) struct MoveUp;
impl OpTrait for MoveUp {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|state, _term| {
            state.screen_mut().select_previous(NavMode::Normal);
            Ok(())
        }))
    }
}

#[derive(Display)]
#[display(fmt = "Down")]
pub(crate) struct MoveDown;
impl OpTrait for MoveDown {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|state, _term| {
            state.screen_mut().select_next(NavMode::Normal);
            Ok(())
        }))
    }
}

#[derive(Display)]
#[display(fmt = "Down line")]
pub(crate) struct MoveDownLine;
impl OpTrait for MoveDownLine {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|state, _term| {
            state.screen_mut().select_next(NavMode::IncludeHunkLines);
            Ok(())
        }))
    }
}

#[derive(Display)]
#[display(fmt = "Up line")]
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

#[derive(Display)]
#[display(fmt = "Next section")]
pub(crate) struct MoveNextSection;
impl OpTrait for MoveNextSection {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|state, _term| {
            let depth = state.screen().get_selected_item().depth;
            state.screen_mut().select_next(NavMode::Siblings { depth });
            Ok(())
        }))
    }
}

#[derive(Display)]
#[display(fmt = "Prev section")]
pub(crate) struct MovePrevSection;
impl OpTrait for MovePrevSection {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|state, _term| {
            let depth = state.screen().get_selected_item().depth;
            state
                .screen_mut()
                .select_previous(NavMode::Siblings { depth });
            Ok(())
        }))
    }
}

#[derive(Display)]
#[display(fmt = "Parent section")]
pub(crate) struct MoveParentSection;
impl OpTrait for MoveParentSection {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|state, _term| {
            let depth = state.screen().get_selected_item().depth.saturating_sub(1);
            state
                .screen_mut()
                .select_previous(NavMode::Siblings { depth });
            Ok(())
        }))
    }
}

#[derive(Display)]
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

#[derive(Display)]
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
