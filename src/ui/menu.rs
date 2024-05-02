use std::borrow::Cow;

use super::SizedWidget;
use crate::{
    bindings::Bindings,
    config::Config,
    items::Item,
    menu::{arg::Arg, PendingMenu},
    ops::Op,
};
use itertools::{EitherOrBoth, Itertools};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::Style,
    text::{Line, Span, Text},
    widgets::{Row, Table, Widget},
};

pub(crate) struct MenuWidget<'a> {
    pending_binds_rows: Vec<Row<'a>>,
}

impl<'a> MenuWidget<'a> {
    pub fn new(
        config: &Config,
        bindings: &'a Bindings,
        pending: &'a PendingMenu,
        item: &'a Item,
    ) -> SizedWidget<Self> {
        let style = &config.style;

        let arg_binds = bindings.arg_list(pending).collect::<Vec<_>>();

        let pending_binds_title = Line::styled(format!("{}", pending.menu), &style.command);
        let pending_binds_rows = bindings
            .list(&pending.menu)
            .filter(|keybind| !keybind.op.clone().implementation().is_target_op())
            .group_by(|bind| &bind.op)
            .into_iter()
            .filter(|(op, _binds)| !matches!(op, Op::OpenMenu(_)))
            .map(|(op, binds)| {
                Row::new(vec![
                    Text::styled(
                        binds.into_iter().map(|bind| &bind.raw).join("/"),
                        &style.hotkey,
                    )
                    .right_aligned(),
                    Text::styled(format!("{}", op.clone().implementation()), Style::new()),
                ])
            })
            .collect::<Vec<_>>();

        let menus = bindings
            .list(&pending.menu)
            .filter(|keybind| !keybind.op.clone().implementation().is_target_op())
            .filter(|bind| matches!(bind.op, Op::OpenMenu(_)))
            .collect::<Vec<_>>();

        let menu_binds_column = if !menus.is_empty() {
            Some(Line::styled("Submenu", &style.command))
        } else {
            None
        }
        .into_iter()
        .chain(menus.into_iter().map(|bind| {
            let Op::OpenMenu(menu) = bind.op else {
                unreachable!();
            };

            Line::from(vec![
                Span::styled(&bind.raw, &style.hotkey),
                Span::styled(format!(" {}", menu), Style::new()),
            ])
        }))
        .collect::<Vec<_>>();

        let mut right_column = vec![];
        if let Some(target_data) = &item.target_data {
            let target_binds = bindings
                .list(&pending.menu)
                .filter(|keybind| keybind.op.clone().implementation().is_target_op())
                .filter(|keybind| {
                    keybind
                        .op
                        .clone()
                        .implementation()
                        .get_action(Some(target_data))
                        .is_some()
                })
                .collect::<Vec<_>>();

            if !target_binds.is_empty() {
                right_column.push(item.display.clone());
            }

            for bind in target_binds {
                right_column.push(Line::from(vec![
                    Span::styled(&bind.raw, &style.hotkey),
                    Span::styled(
                        format!(" {}", bind.op.clone().implementation()),
                        Style::new(),
                    ),
                ]));
            }
        }

        if !arg_binds.is_empty() {
            right_column.push(Line::styled("Arguments", &style.command));
        }

        for bind in arg_binds {
            let Op::ToggleArg(name) = &bind.op else {
                unreachable!();
            };

            let on = pending
                .args
                .get(name.as_str())
                .map(Arg::is_acive)
                .unwrap_or(false);

            right_column.push(Line::from(vec![
                Span::styled(&bind.raw, &style.hotkey),
                Span::raw(" "),
                Span::raw(pending.args.get(&Cow::from(name)).unwrap().display),
                Span::raw(" ("),
                Span::styled(
                    format!("{}", bind.op.clone().implementation()),
                    if on {
                        Style::from(&style.active_arg)
                    } else {
                        Style::new()
                    },
                ),
                Span::raw(")"),
            ]));
        }

        SizedWidget {
            height: 1 + pending_binds_rows.len() as u16,
            widget: Self { pending_binds_rows },
        }
    }
}

impl<'a> Widget for MenuWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let widths = [
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Fill(1),
        ];

        let table = Table::new(self.pending_binds_rows, widths);
        Widget::render(table, area, buf)
    }
}
