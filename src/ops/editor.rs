use super::{Action, OpTrait};
use crate::{
    app::{root_menu, App, PromptParams, State},
    menu::PendingMenu,
    screen::NavMode,
    item_data::ItemData,
    term::Term,
};
use std::rc::Rc;

pub(crate) struct Quit;
impl OpTrait for Quit {
    fn get_action(&self, _target: Option<&ItemData>) -> Option<Action> {
        Some(Rc::new(|app, term| {
            let menu = app
                .state
                .pending_menu
                .as_ref()
                .map(|pending_menu| pending_menu.menu);

            if menu == root_menu(&app.state.config) {
                if app.state.screens.len() == 1 {
                    if app.state.config.general.confirm_quit.enabled {
                        app.confirm(term, "Really quit? (y or n)")?;
                    };

                    app.state.quit = true;
                } else {
                    app.state.screens.pop();
                    if let Some(screen) = app.state.screens.last_mut() {
                        screen.update()?;
                    }
                }
            } else {
                app.close_menu();
                return Ok(());
            }

            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "Quit/Close".into()
    }
}

pub(crate) struct OpenMenu(pub crate::menu::Menu);
impl OpTrait for OpenMenu {
    fn get_action(&self, _target: Option<&ItemData>) -> Option<Action> {
        let submenu = self.0;
        Some(Rc::new(move |app, _term| {
            app.state.pending_menu = Some(PendingMenu::init(submenu));
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "Submenu".into()
    }
}

pub(crate) struct Refresh;
impl OpTrait for Refresh {
    fn get_action(&self, _target: Option<&ItemData>) -> Option<Action> {
        Some(Rc::new(|app, _term| {
            app.close_menu();
            app.screen_mut().update()
        }))
    }

    fn display(&self, _state: &State) -> String {
        "Refresh".into()
    }
}

pub(crate) struct ToggleArg(pub String);
impl OpTrait for ToggleArg {
    fn get_action(&self, _target: Option<&ItemData>) -> Option<Action> {
        let arg_name = self.0.clone();
        Some(Rc::new(move |app, term| {
            let mut need_prompt = None;
            let mut default = None;

            let maybe_entry = if let Some(menu) = &mut app.state.pending_menu {
                Some(menu.args.entry(arg_name.clone().into()))
            } else {
                None
            };

            if let Some(entry) = maybe_entry {
                entry.and_modify(|arg| {
                    if arg.is_active() {
                        arg.unset();
                    } else if arg.expects_value() {
                        default = arg.default_as_string();
                        need_prompt = Some(arg.display);
                    } else {
                        arg.set("").expect("Should succeed");
                    }
                });
            }

            let arg_name = arg_name.clone();
            let parse_and_set_arg =
                Box::new(move |app: &mut App, _term: &mut Term, value: &str| {
                    if let Some(menu) = &mut app.state.pending_menu {
                        if let Some(entry) = menu.args.get_mut(arg_name.as_str()) {
                            return entry.set(value);
                        }
                    }

                    Ok(())
                });

            if let Some(display) = need_prompt {
                let arg = app.prompt(
                    term,
                    &PromptParams {
                        prompt: display,
                        create_default_value: Box::new(move |_| default.clone()),
                        hide_menu: false,
                    },
                )?;

                parse_and_set_arg(app, term, &arg)?;
            }

            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        self.0.clone()
    }
}

pub(crate) struct ToggleSection;
impl OpTrait for ToggleSection {
    fn get_action(&self, _target: Option<&ItemData>) -> Option<Action> {
        Some(Rc::new(|app, _term| {
            app.close_menu();
            app.screen_mut().toggle_section();
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "Toggle section".into()
    }
}

pub(crate) struct MoveUp;
impl OpTrait for MoveUp {
    fn get_action(&self, _target: Option<&ItemData>) -> Option<Action> {
        Some(Rc::new(|app, _term| {
            app.close_menu();
            app.screen_mut().select_previous(NavMode::Normal);
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "Up".into()
    }
}

pub(crate) struct MoveDown;
impl OpTrait for MoveDown {
    fn get_action(&self, _target: Option<&ItemData>) -> Option<Action> {
        Some(Rc::new(|app, _term| {
            app.close_menu();
            app.screen_mut().select_next(NavMode::Normal);
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "Down".into()
    }
}

pub(crate) struct MoveDownLine;
impl OpTrait for MoveDownLine {
    fn get_action(&self, _target: Option<&ItemData>) -> Option<Action> {
        Some(Rc::new(|app, _term| {
            app.close_menu();
            app.screen_mut().select_next(NavMode::IncludeHunkLines);
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "Down line".into()
    }
}

pub(crate) struct MoveUpLine;
impl OpTrait for MoveUpLine {
    fn get_action(&self, _target: Option<&ItemData>) -> Option<Action> {
        Some(Rc::new(|app, _term| {
            app.close_menu();
            app.screen_mut().select_previous(NavMode::IncludeHunkLines);
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "Up line".into()
    }
}

pub(crate) struct MoveNextSection;
impl OpTrait for MoveNextSection {
    fn get_action(&self, _target: Option<&ItemData>) -> Option<Action> {
        Some(Rc::new(|app, _term| {
            app.close_menu();
            let depth = app.screen().get_selected_item().depth;
            app.screen_mut().select_next(NavMode::Siblings { depth });
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "Next section".into()
    }
}

pub(crate) struct MovePrevSection;
impl OpTrait for MovePrevSection {
    fn get_action(&self, _target: Option<&ItemData>) -> Option<Action> {
        Some(Rc::new(|app, _term| {
            app.close_menu();
            let depth = app.screen().get_selected_item().depth;
            app.screen_mut()
                .select_previous(NavMode::Siblings { depth });
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "Prev section".into()
    }
}

pub(crate) struct MoveParentSection;
impl OpTrait for MoveParentSection {
    fn get_action(&self, _target: Option<&ItemData>) -> Option<Action> {
        Some(Rc::new(|app, _term| {
            app.close_menu();
            let depth = app.screen().get_selected_item().depth.saturating_sub(1);
            app.screen_mut()
                .select_previous(NavMode::Siblings { depth });
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "Parent section".into()
    }
}

pub(crate) struct HalfPageUp;
impl OpTrait for HalfPageUp {
    fn get_action(&self, _target: Option<&ItemData>) -> Option<Action> {
        Some(Rc::new(|app, _term| {
            app.close_menu();
            app.screen_mut().scroll_half_page_up();
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "Half page up".into()
    }
}

pub(crate) struct HalfPageDown;
impl OpTrait for HalfPageDown {
    fn get_action(&self, _target: Option<&ItemData>) -> Option<Action> {
        Some(Rc::new(|app, _term| {
            app.close_menu();
            app.screen_mut().scroll_half_page_down();
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "Half page down".into()
    }
}
