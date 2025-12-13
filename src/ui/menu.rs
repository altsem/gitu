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

    let config = Arc::clone(&state.config);
    let item = state.screens.last().unwrap().get_selected_item();
    let style = &config.style;

    let arg_binds = config.bindings.arg_list(pending).collect::<Vec<_>>();

    let non_target_binds = config
        .bindings
        .list(&pending.menu)
        .filter(|keybind| !keybind.op.clone().implementation().is_target_op())
        .collect::<Vec<_>>();

    let menus = non_target_binds
        .iter()
        .filter(|bind| matches!(bind.op, Op::OpenMenu(_)))
        .collect::<Vec<_>>();

    let target_binds = config
        .bindings
        .list(&pending.menu)
        .filter(|keybind| keybind.op.clone().implementation().is_target_op())
        .filter(|keybind| {
            keybind
                .op
                .clone()
                .implementation()
                .get_action(&item.data)
                .is_some()
        })
        .collect::<Vec<_>>();

    let line = item.to_line(Arc::clone(&config));
    let separator_style = Style::from(&style.separator);

    layout.vertical(None, OPTS, |layout| {
        ui::repeat_chars(layout, width, ui::DASHES, separator_style);

        layout.horizontal(None, OPTS.gap(3).pad(1), |layout| {
            layout.vertical(None, OPTS, |layout| {
                layout_line(
                    layout,
                    Line::styled(format!("{}", pending.menu), &style.menu.heading),
                );

                for (op, binds) in non_target_binds
                    .iter()
                    .chunk_by(|bind| &bind.op)
                    .into_iter()
                    .filter(|(op, _binds)| !matches!(op, Op::OpenMenu(_)))
                {
                    super::layout_line(
                        layout,
                        Line::from(vec![
                            Span::styled(
                                binds.into_iter().map(|bind| &bind.raw).join("/"),
                                &style.menu.key,
                            ),
                            Span::styled(
                                format!(" {}", op.clone().implementation().display(state)),
                                Style::new(),
                            ),
                        ]),
                    );
                }
            });

            layout.vertical(None, OPTS, |layout| {
                if !menus.is_empty() {
                    super::layout_line(layout, Line::styled("Submenu", &style.menu.heading));
                }

                for (op, binds) in menus.iter().chunk_by(|bind| &bind.op).into_iter() {
                    let Op::OpenMenu(menu) = op else {
                        unreachable!();
                    };

                    super::layout_line(
                        layout,
                        Line::from(vec![
                            Span::styled(
                                binds.into_iter().map(|bind| &bind.raw).join("/"),
                                &style.menu.key,
                            ),
                            Span::styled(format!(" {menu}"), Style::new()),
                        ]),
                    );
                }
            });

            layout.vertical(None, OPTS, |layout| {
                if !target_binds.is_empty() {
                    super::layout_line(layout, line);
                }

                for bind in target_binds {
                    super::layout_line(
                        layout,
                        Line::from(vec![
                            Span::styled(bind.raw.clone(), &style.menu.key),
                            Span::styled(
                                format!(" {}", bind.op.clone().implementation().display(state)),
                                Style::new(),
                            ),
                        ]),
                    );
                }

                if !arg_binds.is_empty() {
                    super::layout_line(layout, Line::styled("Arguments", &style.menu.heading));
                }

                for bind in arg_binds {
                    let Op::ToggleArg(name) = &bind.op else {
                        unreachable!();
                    };

                    let arg = pending.args.get(name.as_str()).unwrap();

                    super::layout_line(
                        layout,
                        Line::from(vec![
                            Span::styled(bind.raw.clone(), &style.menu.key),
                            Span::raw(format!(" {} (", arg.display)),
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
                    );
                }
            });
        });
    });
}
