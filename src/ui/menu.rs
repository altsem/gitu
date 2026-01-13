use std::sync::Arc;

use crate::ui::layout::OPTS;
use crate::ui::{self, UiTree};
use crate::{app::State, ops::Op, ui::layout_line};
use itertools::Itertools;
use ratatui::{
    style::Style,
    text::{Line, Span},
};

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

    let config = Arc::clone(&state.config);
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

    let line = item.to_line(Arc::clone(&config));
    let separator_style = Style::from(&style.separator);

    layout.vertical(None, OPTS, |layout| {
        ui::repeat_chars(layout, width, ui::DASHES, separator_style);

        layout.horizontal(None, OPTS.gap(3).pad(1), |layout| {
            // Column 1: Main menu commands
            if !non_menu_binds.is_empty() {
                layout.vertical(None, OPTS, |layout| {
                    layout_line(
                        layout,
                        Line::styled(format!("{}", pending.menu), &style.menu.heading),
                    );

                    layout_keybinds_table(
                        layout,
                        non_menu_binds
                            .into_iter()
                            .map(|(op, binds)| {
                                (
                                    Line::styled(
                                        binds.iter().map(|bind| bind.raw.as_str()).join("/"),
                                        &style.menu.key,
                                    ),
                                    Line::raw(op.clone().implementation().display(state)),
                                )
                            })
                            .collect(),
                    );
                });
            }

            // Column 2: Submenus
            if !menu_binds.is_empty() {
                layout.vertical(None, OPTS, |layout| {
                    layout_line(layout, Line::styled("Submenu", &style.menu.heading));

                    layout_keybinds_table(
                        layout,
                        menu_binds
                            .into_iter()
                            .map(|(op, binds)| {
                                let Op::OpenMenu(menu) = op else {
                                    unreachable!();
                                };
                                (
                                    Line::styled(
                                        binds.iter().map(|bind| bind.raw.as_str()).join("/"),
                                        &style.menu.key,
                                    ),
                                    Line::raw(menu.to_string()),
                                )
                            })
                            .collect(),
                    );
                });
            }

            // Column 3: Target commands and arguments
            layout.vertical(None, OPTS, |layout| {
                if !target_binds.is_empty() {
                    layout_line(layout, line);

                    layout_keybinds_table(
                        layout,
                        target_binds
                            .into_iter()
                            .map(|bind| {
                                (
                                    Line::styled(bind.raw.clone(), &style.menu.key),
                                    Line::raw(bind.op.clone().implementation().display(state)),
                                )
                            })
                            .collect(),
                    );
                }

                if !arg_binds.is_empty() {
                    layout_line(layout, Line::styled("Arguments", &style.menu.heading));

                    layout_keybinds_table(
                        layout,
                        arg_binds
                            .into_iter()
                            .map(|bind| {
                                let Op::ToggleArg(name) = &bind.op else {
                                    unreachable!();
                                };

                                let arg = pending.args.get(name.as_str()).unwrap();

                                (
                                    Line::styled(bind.raw.clone(), &style.menu.key),
                                    Line::from(vec![
                                        Span::raw(arg.display),
                                        Span::raw(" ("),
                                        Span::styled(
                                            arg.get_cli_token().to_string(),
                                            if arg.is_active() {
                                                Style::from(&style.menu.active_arg)
                                            } else {
                                                Style::from(&style.menu.inactive_arg)
                                            },
                                        ),
                                        Span::raw(")"),
                                    ]),
                                )
                            })
                            .collect(),
                    );
                }
            });
        });
    });
}

fn layout_keybinds_table<'a>(layout: &mut UiTree<'a>, items: Vec<(Line<'a>, Line<'a>)>) {
    layout.horizontal(None, OPTS.gap(1), |layout| {
        layout.vertical(None, OPTS, |layout| {
            for (key, _) in items.iter() {
                layout_line(layout, key.clone());
            }
        });

        layout.vertical(None, OPTS, |layout| {
            for (_, value) in items {
                layout_line(layout, value);
            }
        });
    });
}
