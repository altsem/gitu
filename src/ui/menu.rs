use super::SizedWidget;
use crate::{
    bindings::Bindings, config::Config, items::Item, menu::PendingMenu, ops::Op, state::State,
};
use itertools::Itertools;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Row, Table, Widget},
};

pub(crate) struct MenuWidget<'a> {
    table: Table<'a>,
}

impl<'a> MenuWidget<'a> {
    pub fn new(
        config: &Config,
        bindings: &'a Bindings,
        pending: &'a PendingMenu,
        item: &'a Item,
        state: &'a State,
    ) -> SizedWidget<Self> {
        let style = &config.style;

        let arg_binds = bindings.arg_list(pending).collect::<Vec<_>>();

        let non_target_binds = bindings
            .list(&pending.menu)
            .filter(|keybind| !keybind.op.clone().implementation().is_target_op())
            .collect::<Vec<_>>();

        let mut pending_binds_column = vec![];
        pending_binds_column.push(Line::styled(format!("{}", pending.menu), &style.command));
        for (op, binds) in non_target_binds
            .iter()
            .chunk_by(|bind| &bind.op)
            .into_iter()
            .filter(|(op, _binds)| !matches!(op, Op::OpenMenu(_)))
        {
            pending_binds_column.push(Line::from(vec![
                Span::styled(
                    binds.into_iter().map(|bind| &bind.raw).join("/"),
                    &style.hotkey,
                ),
                Span::styled(
                    format!(" {}", op.clone().implementation().display(state)),
                    Style::new(),
                ),
            ]));
        }

        let menus = non_target_binds
            .iter()
            .filter(|bind| matches!(bind.op, Op::OpenMenu(_)))
            .collect::<Vec<_>>();

        let mut menu_binds_column = vec![];
        if !menus.is_empty() {
            menu_binds_column.push(Line::styled("Submenu", &style.command));
        }
        for (op, binds) in menus.iter().chunk_by(|bind| &bind.op).into_iter() {
            let Op::OpenMenu(menu) = op else {
                unreachable!();
            };

            menu_binds_column.push(Line::from(vec![
                Span::styled(
                    binds.into_iter().map(|bind| &bind.raw).join("/"),
                    &style.hotkey,
                ),
                Span::styled(format!(" {}", menu), Style::new()),
            ]));
        }

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
                        format!(" {}", bind.op.clone().implementation().display(state)),
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

            let arg = pending.args.get(name.as_str()).unwrap();

            right_column.push(Line::from(vec![
                Span::styled(&bind.raw, &style.hotkey),
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
            ]));
        }

        let widths = [
            col_width(&pending_binds_column),
            col_width(&menu_binds_column),
            Constraint::Fill(1),
        ];

        let columns = [pending_binds_column, menu_binds_column, right_column];

        let max_rows = columns.iter().map(Vec::len).max().unwrap_or(0);
        let rows = (0..(max_rows)).map(|i| {
            Row::new(
                columns
                    .iter()
                    .map(|col| col.get(i).cloned().unwrap_or(Line::raw(""))),
            )
        });

        let (lines, table) = (rows.len(), Table::new(rows, widths).column_spacing(3));

        SizedWidget {
            height: 1 + lines as u16,
            widget: MenuWidget {
                table: table.block(super::popup_block()),
            },
        }
    }
}

fn col_width(column: &[Line<'_>]) -> Constraint {
    Constraint::Length(column.iter().map(|line| line.width()).max().unwrap_or(0) as u16)
}

impl<'a> Widget for MenuWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        Widget::render(self.table, area, buf)
    }
}
