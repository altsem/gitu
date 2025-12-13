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

    layout.vertical(None, OPTS, |layout| {
        ui::repeat_chars(layout, width, ui::DASHES, ui::STYLE);

        layout.horizontal(None, OPTS.gap(3).pad(1), |layout| {
            layout.vertical(None, OPTS, |layout| {
                layout_line(
                    layout,
                    Line::styled(format!("{}", pending.menu), &style.command),
                );

                let non_menu_binds: Vec<_> = non_target_binds
                    .iter()
                    .filter(|bind| !matches!(bind.op, Op::OpenMenu(_)))
                    .chunk_by(|bind| &bind.op)
                    .into_iter()
                    .map(|(op, binds)| {
                        let binds: Vec<_> = binds.collect();
                        (op, binds)
                    })
                    .collect();

                let max_key_width = non_menu_binds
                    .iter()
                    .map(|(_op, binds)| {
                        binds.iter().map(|bind| bind.raw.len()).sum::<usize>()
                            + binds.len().saturating_sub(1)
                    })
                    .max()
                    .unwrap_or(0);

                for (op, binds) in non_menu_binds {
                    let key_str = format_key_string_from_double_refs(&binds);
                    let padding = calc_padding(key_str.len(), max_key_width);
                    super::layout_line(
                        layout,
                        Line::from(vec![
                            Span::styled(key_str, &style.hotkey),
                            Span::raw(format!(
                                "{} {}",
                                padding,
                                op.clone().implementation().display(state)
                            )),
                        ]),
                    );
                }
            });

            layout.vertical(None, OPTS, |layout| {
                if !menus.is_empty() {
                    super::layout_line(layout, Line::styled("Submenu", &style.command));
                }

                let menu_binds: Vec<_> = menus
                    .iter()
                    .chunk_by(|bind| &bind.op)
                    .into_iter()
                    .map(|(op, binds)| {
                        let binds: Vec<_> = binds.collect();
                        (op, binds)
                    })
                    .collect();

                let max_menu_key_width = menu_binds
                    .iter()
                    .map(|(_op, binds)| {
                        binds.iter().map(|bind| bind.raw.len()).sum::<usize>()
                            + binds.len().saturating_sub(1)
                    })
                    .max()
                    .unwrap_or(0);

                for (op, binds) in menu_binds {
                    let Op::OpenMenu(menu) = op else {
                        unreachable!();
                    };

                    let key_str = format_key_string_from_triple_refs(&binds);
                    let padding = calc_padding(key_str.len(), max_menu_key_width);
                    super::layout_line(
                        layout,
                        Line::from(vec![
                            Span::styled(key_str, &style.hotkey),
                            Span::raw(format!("{} {}", padding, menu)),
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
                            Span::styled(bind.raw.clone(), &style.hotkey),
                            Span::styled(
                                format!(" {}", bind.op.clone().implementation().display(state)),
                                Style::new(),
                            ),
                        ]),
                    );
                }

                if !arg_binds.is_empty() {
                    super::layout_line(layout, Line::styled("Arguments", &style.command));
                }

                for bind in arg_binds {
                    let Op::ToggleArg(name) = &bind.op else {
                        unreachable!();
                    };

                    let arg = pending.args.get(name.as_str()).unwrap();

                    super::layout_line(
                        layout,
                        Line::from(vec![
                            Span::styled(bind.raw.clone(), &style.hotkey),
                            Span::raw(" "),
                            Span::raw(arg.display),
                            Span::raw(" ("),
                            Span::styled(
                                arg.get_cli_token().to_string(),
                                if arg.is_active() {
                                    Style::from(&style.active_arg)
                                } else {
                                    Style::new()
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

macro_rules! format_key_string {
    ($binds:expr) => {
        $binds.iter().map(|bind| bind.raw.as_str()).join("/")
    };
}

fn format_key_string_from_double_refs(binds: &[&&crate::bindings::Binding]) -> String {
    format_key_string!(binds)
}

fn format_key_string_from_triple_refs(binds: &[&&&crate::bindings::Binding]) -> String {
    format_key_string!(binds)
}

fn calc_padding(key_str_len: usize, max_width: usize) -> String {
    " ".repeat(max_width.saturating_sub(key_str_len))
}
