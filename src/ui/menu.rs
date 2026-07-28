use std::borrow::Cow;

use crate::menu::arg::Arg;
use crate::style::Style;
use crate::ui::item::layout_item;
use crate::ui::layout::opts;
use crate::ui::{self, UiTree, layout_line, layout_span, repeat_chars};
use crate::{app::State, config::Config, ops::Op};
use itertools::Itertools;
use unicode_width::UnicodeWidthStr;

/// The value column of a keybind table row.
enum MenuValue<'a> {
    Text(Cow<'a, str>),
    Arg(&'a Arg),
}

pub(crate) fn layout_menu<'a>(layout: &mut UiTree<'a>, state: &'a State, width: usize) {
    let Some(ref pending) = state.pending_menu else {
        return;
    };

    if pending.is_hidden {
        return;
    }

    if state.picker.is_some() {
        return;
    }

    let config = &state.config;
    let item = state.screens.last().unwrap().get_selected_item();
    let style = &config.style;

    let arg_binds = config.bindings.arg_list(pending).collect::<Vec<_>>();
    let (target_binds, non_target_binds): (Vec<_>, Vec<_>) = config
        .bindings
        .list(&pending.menu)
        .partition(|keybind| keybind.op.clone().implementation().is_target_op());
    let target_binds: Vec<_> = target_binds
        .into_iter()
        .filter(|keybind| {
            keybind
                .op
                .clone()
                .implementation()
                .get_action(&item.data)
                .is_some()
        })
        .collect();
    let (menu_binds, non_menu_binds): (Vec<_>, Vec<_>) = non_target_binds
        .into_iter()
        .chunk_by(|bind| &bind.op)
        .into_iter()
        .map(|(op, binds)| {
            let binds: Vec<_> = binds.collect();
            (op, binds)
        })
        .partition(|(op, _binds)| matches!(op, Op::OpenMenu(_)));

    let separator_style = Style::from(&style.separator);

    layout.vertical(None, opts(), |layout| {
        ui::repeat_chars(layout, width, ui::DASHES, separator_style);

        layout.horizontal(None, opts().gap(3).pad(1), |layout| {
            // Column 1: Main menu commands
            if !non_menu_binds.is_empty() {
                layout.vertical(None, opts(), |layout| {
                    layout_line(
                        layout,
                        pending.menu.to_string().into(),
                        Style::from(&style.menu.heading),
                    );

                    layout_keybinds_table(
                        layout,
                        config,
                        non_menu_binds
                            .into_iter()
                            .map(|(op, binds)| {
                                (
                                    binds.iter().map(|bind| bind.raw.as_str()).join("/").into(),
                                    MenuValue::Text(
                                        op.clone().implementation().display(state).into(),
                                    ),
                                )
                            })
                            .collect(),
                    );
                });
            }

            // Column 2: Submenus
            if !menu_binds.is_empty() {
                layout.vertical(None, opts(), |layout| {
                    layout_line(layout, "Submenu".into(), Style::from(&style.menu.heading));

                    layout_keybinds_table(
                        layout,
                        config,
                        menu_binds
                            .into_iter()
                            .map(|(op, binds)| {
                                let Op::OpenMenu(menu) = op else {
                                    unreachable!();
                                };

                                (
                                    binds.iter().map(|bind| bind.raw.as_str()).join("/").into(),
                                    MenuValue::Text(menu.to_string().into()),
                                )
                            })
                            .collect(),
                    );
                });
            }

            // Column 3: Target commands and arguments
            layout.vertical(None, opts(), |layout| {
                if !target_binds.is_empty() {
                    layout.horizontal(None, opts(), |layout| {
                        layout_item(layout, item, config, Style::new());
                    });

                    layout_keybinds_table(
                        layout,
                        config,
                        target_binds
                            .into_iter()
                            .map(|bind| {
                                (
                                    bind.raw.as_str().into(),
                                    MenuValue::Text(
                                        bind.op.clone().implementation().display(state).into(),
                                    ),
                                )
                            })
                            .collect(),
                    );
                }

                if !arg_binds.is_empty() {
                    layout_line(layout, "Arguments".into(), Style::from(&style.menu.heading));

                    layout_keybinds_table(
                        layout,
                        config,
                        arg_binds
                            .into_iter()
                            .map(|bind| {
                                let Op::ToggleArg(name) = &bind.op else {
                                    unreachable!();
                                };

                                let arg = pending.args.get(name.as_str()).unwrap();

                                (bind.raw.as_str().into(), MenuValue::Arg(arg))
                            })
                            .collect(),
                    );
                }
            });
        });
    });
}

fn layout_keybinds_table<'a>(
    layout: &mut UiTree<'a>,
    config: &Config,
    rows: Vec<(Cow<'a, str>, MenuValue<'a>)>,
) {
    const SPACES: &str = "                                                                ";
    let key_style = Style::from(&config.style.menu.key);
    let max_width = rows
        .iter()
        .map(|(key, _)| UnicodeWidthStr::width(key.as_ref()))
        .max()
        .unwrap_or(0)
        + 1;

    layout.vertical(None, opts(), |layout| {
        for (key, value) in rows {
            let padding = max_width - UnicodeWidthStr::width(key.as_ref());

            layout.horizontal(None, opts(), |layout| {
                layout_line(layout, key, key_style);
                repeat_chars(layout, padding, SPACES, Style::new());

                layout.horizontal(None, opts(), |layout| match value {
                    MenuValue::Text(text) => layout_span(layout, (text, Style::new())),
                    MenuValue::Arg(arg) => {
                        layout_span(layout, (arg.display.into(), Style::new()));
                        layout_span(layout, (" (".into(), Style::new()));
                        layout_span(
                            layout,
                            (
                                arg.get_cli_token().into(),
                                if arg.is_active() {
                                    Style::from(&config.style.menu.active_arg)
                                } else {
                                    Style::from(&config.style.menu.inactive_arg)
                                },
                            ),
                        );
                        layout_span(layout, (")".into(), Style::new()));
                    }
                });
            });
        }
    });
}
