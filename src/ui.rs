use std::sync::Arc;
use std::sync::RwLock;

use crate::config::Config;
use crate::items::Item;
use crate::keybinds;
use crate::keybinds::Keybind;
use crate::menu::PendingMenu;
use crate::ops::Op;
use crate::state::State;
use crate::CmdLogEntry;
use itertools::EitherOrBoth;
use itertools::Itertools;
use ratatui::prelude::*;
use ratatui::style::Stylize;
use ratatui::widgets::*;
use ratatui::Frame;
use tui_prompts::State as _;
use tui_prompts::TextPrompt;

pub(crate) fn ui(frame: &mut Frame, state: &mut State) {
    let (log_len, maybe_log): (usize, Option<Paragraph>) =
        if !state.current_cmd_log_entries.is_empty() {
            let text: Text = state
                .current_cmd_log_entries
                .iter()
                .flat_map(|cmd| format_command(&state.config, cmd))
                .collect::<Vec<_>>()
                .into();

            (
                text.lines.len() + 1,
                Some(Paragraph::new(text).block(popup_block())),
            )
        } else {
            (0, None)
        };

    let (menu_len, maybe_menu) = if let Some(ref menu) = state.pending_menu {
        let (lines, table) =
            format_keybinds_menu(&state.config, menu, state.screen().get_selected_item());

        (lines + 1, Some(table.block(popup_block())))
    } else {
        (0, None)
    };

    let layout = Layout::new(
        Direction::Vertical,
        [
            Constraint::Min(1),
            Constraint::Length(log_len as u16),
            Constraint::Length(menu_len as u16),
            Constraint::Length(if state.prompt.data.is_some() { 2 } else { 0 }),
        ],
    )
    .split(frame.size());

    frame.render_widget(state.screen(), layout[0]);

    if let Some(log) = maybe_log {
        frame.render_widget(log, layout[1]);
    }
    if let Some(menu) = maybe_menu {
        frame.render_widget(menu, layout[2]);
    }

    if let Some(prompt_data) = &state.prompt.data {
        let prompt = TextPrompt::new(prompt_data.prompt_text.clone()).with_block(popup_block());
        frame.render_stateful_widget(prompt, layout[3], &mut state.prompt.state);
        let (cx, cy) = state.prompt.state.cursor();
        frame.set_cursor(cx, cy);
    }
}

fn format_command<'a>(config: &Config, log: &Arc<RwLock<CmdLogEntry>>) -> Vec<Line<'a>> {
    match &*log.read().unwrap() {
        CmdLogEntry::Cmd { args, out } => [Line::styled(
            format!("$ {}{}", args, if out.is_some() { "" } else { "..." }),
            &config.style.command,
        )]
        .into_iter()
        .chain(out.iter().flat_map(|out| Text::raw(out.to_string()).lines))
        .collect::<Vec<_>>(),
        CmdLogEntry::Error(err) => {
            vec![Line::styled(
                format!("! {}", err),
                Style::new().red().bold(),
            )]
        }
    }
}

fn format_keybinds_menu<'b>(
    config: &Config,
    pending: &'b PendingMenu,
    item: &'b Item,
) -> (usize, Table<'b>) {
    let style = &config.style;

    let arg_binds = keybinds::arg_list(&pending.menu).collect::<Vec<_>>();

    let non_target_binds = keybinds::list(&pending.menu)
        .filter(|keybind| !keybind.op.implementation().is_target_op())
        .collect::<Vec<_>>();

    let mut pending_binds_column = vec![];
    pending_binds_column.push(Line::styled(format!("{}", pending.menu), &style.command));
    for (op, binds) in non_target_binds
        .iter()
        .group_by(|bind| bind.op)
        .into_iter()
        .filter(|(op, _binds)| !matches!(op, Op::Menu(_)))
    {
        pending_binds_column.push(Line::from(vec![
            Span::styled(
                binds
                    .into_iter()
                    .map(|bind| Keybind::format_key(bind))
                    .join(" "),
                &style.hotkey,
            ),
            Span::styled(format!(" {}", op.implementation()), Style::new()),
        ]));
    }

    let menus = non_target_binds
        .iter()
        .filter(|bind| matches!(bind.op, Op::Menu(_)))
        .collect::<Vec<_>>();

    let mut menu_binds_column = vec![];
    if !menus.is_empty() {
        menu_binds_column.push(Line::styled("Submenu", &style.command));
    }
    for bind in menus {
        let Op::Menu(menu) = bind.op else {
            unreachable!();
        };

        menu_binds_column.push(Line::from(vec![
            Span::styled(Keybind::format_key(bind), &style.hotkey),
            Span::styled(format!(" {}", menu), Style::new()),
        ]));
    }

    let mut right_column = vec![];
    if let Some(target_data) = &item.target_data {
        let target_binds = keybinds::list(&pending.menu)
            .filter(|keybind| keybind.op.implementation().is_target_op())
            .filter(|keybind| {
                keybind
                    .op
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
                Span::styled(Keybind::format_key(bind), &style.hotkey),
                Span::styled(format!(" {}", bind.op.implementation()), Style::new()),
            ]));
        }
    }

    if !arg_binds.is_empty() {
        right_column.push(Line::styled("Arguments", &style.command));
    }

    for bind in arg_binds {
        let Op::ToggleArg(name) = bind.op else {
            unreachable!();
        };

        let on = *pending.args.get(name).unwrap_or(&false);

        right_column.push(Line::from(vec![
            Span::styled(Keybind::format_key(bind), &style.hotkey),
            Span::styled(
                format!(
                    " {} ({})",
                    bind.op.implementation(),
                    if on { "on" } else { "off" }
                ),
                if on { Style::new() } else { Style::new().dim() },
            ),
        ]));
    }

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

    let widths = [
        Constraint::Max(28),
        Constraint::Max(12),
        Constraint::Length(30),
    ];

    (rows.len(), Table::new(rows, widths))
}

fn popup_block() -> Block<'static> {
    Block::new()
        .borders(Borders::TOP)
        .border_style(Style::new().dim())
        .border_type(ratatui::widgets::BorderType::Plain)
}
