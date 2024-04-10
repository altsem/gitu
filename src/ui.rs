use crate::bindings::Bindings;
use crate::config::Config;
use crate::items::Item;
use crate::menu::arg::Arg;
use crate::menu::PendingMenu;
use crate::ops::Op;
use crate::prompt::Prompt;
use crate::state::State;
use itertools::EitherOrBoth;
use itertools::Itertools;
use ratatui::prelude::*;
use ratatui::style::Stylize;
use ratatui::widgets::*;
use ratatui::Frame;
use std::borrow::Cow;
use tui_prompts::State as _;
use tui_prompts::TextPrompt;

pub(crate) fn ui(frame: &mut Frame, state: &mut State) {
    let (log_len, maybe_log): (u16, Option<Paragraph>) = if !state.current_cmd_log.is_empty() {
        let text: Text = state.current_cmd_log.format_log(&state.config);

        (
            1 + text.lines.len() as u16,
            Some(Paragraph::new(text).block(popup_block())),
        )
    } else {
        (0, None)
    };

    let State {
        screens,
        prompt:
            Prompt {
                data: maybe_prompt_data,
                state: prompt_state,
            },
        pending_menu,
        ..
    } = state;

    let (menu_len, maybe_menu) = if let Some(ref menu) = pending_menu {
        let (lines, table) = format_keybinds_menu(
            &state.config,
            &state.bindings,
            menu,
            screens.last().unwrap().get_selected_item(),
        );

        (1 + lines as u16, Some(table.block(popup_block())))
    } else {
        (0, None)
    };

    let (prompt_len, maybe_prompt) = if let Some(prompt_data) = maybe_prompt_data {
        let prompt = TextPrompt::new(prompt_data.prompt_text.clone());
        (2, Some(prompt.with_block(popup_block())))
    } else {
        (0, None)
    };

    let layout = Layout::new(
        Direction::Vertical,
        [
            Constraint::Min(1),
            Constraint::Length(prompt_len),
            Constraint::Length(menu_len),
            Constraint::Length(log_len),
        ],
    )
    .split(frame.size());

    frame.render_widget(screens.last().unwrap(), layout[0]);

    if let Some(prompt) = maybe_prompt {
        frame.render_stateful_widget(prompt, layout[1], prompt_state);
        let (cx, cy) = state.prompt.state.cursor();
        frame.set_cursor(cx, cy);
    }

    if let Some(menu) = maybe_menu {
        frame.render_widget(menu, layout[2]);
    }

    if let Some(log) = maybe_log {
        frame.render_widget(log, layout[3]);
    }

    screens.last_mut().unwrap().size = layout[0];
}

fn format_keybinds_menu<'b>(
    config: &Config,
    bindings: &'b Bindings,
    pending: &'b PendingMenu,
    item: &'b Item,
) -> (usize, Table<'b>) {
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
        .group_by(|bind| &bind.op)
        .into_iter()
        .filter(|(op, _binds)| !matches!(op, Op::OpenMenu(_)))
    {
        pending_binds_column.push(Line::from(vec![
            Span::styled(
                binds.into_iter().map(|bind| &bind.raw).join("/"),
                &style.hotkey,
            ),
            Span::styled(format!(" {}", op.clone().implementation()), Style::new()),
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
    for bind in menus {
        let Op::OpenMenu(menu) = bind.op else {
            unreachable!();
        };

        menu_binds_column.push(Line::from(vec![
            Span::styled(&bind.raw, &style.hotkey),
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

    let widths = [
        col_width(&pending_binds_column),
        col_width(&menu_binds_column),
        Constraint::Fill(1),
    ];

    let rows = pending_binds_column
        .into_iter()
        .zip_longest(menu_binds_column)
        .zip_longest(right_column)
        .map(|lines| {
            let (ab, c) = lines.or(
                EitherOrBoth::Both(Line::raw(""), Line::raw("")),
                Line::raw(""),
            );
            let (a, b) = ab.or(Line::raw(""), Line::raw(""));

            Row::new([a, b, c])
        })
        .collect::<Vec<_>>();

    (rows.len(), Table::new(rows, widths).column_spacing(3))
}

fn col_width(column: &[Line<'_>]) -> Constraint {
    Constraint::Length(column.iter().map(|line| line.width()).max().unwrap_or(0) as u16)
}

fn popup_block() -> Block<'static> {
    Block::new()
        .borders(Borders::TOP)
        .border_style(Style::new().dim())
        .border_type(ratatui::widgets::BorderType::Plain)
}
