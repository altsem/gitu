use crate::config::Config;
use crate::items::Item;
use crate::keybinds;
use crate::keybinds::Keybind;
use crate::ops::Op;
use crate::ops::OpTrait;
use crate::ops::SubmenuOp;
use crate::state::State;
use crate::CmdMetaBuffer;
use itertools::EitherOrBoth;
use itertools::Itertools;
use ratatui::prelude::*;
use ratatui::style::Stylize;
use ratatui::widgets::*;
use ratatui::Frame;
use tui_prompts::State as _;
use tui_prompts::TextPrompt;

enum Popup<'a> {
    None,
    Paragraph(Paragraph<'a>),
    Table(Table<'a>),
}

pub(crate) fn ui(frame: &mut Frame, state: &mut State) {
    let (popup_line_count, popup): (usize, Popup) = if let Some(ref error) = state.error_buffer {
        let text = error.0.clone().red().bold();
        (1, command_popup(text.into()))
    } else if let Some(ref cmd) = state.cmd_meta_buffer {
        let text = format_command(&state.config, cmd);
        (text.lines.len(), command_popup(text))
    } else if state.pending_submenu_op != SubmenuOp::None {
        format_keybinds_menu(
            &state.config,
            &state.pending_submenu_op,
            state.screen().get_selected_item(),
        )
    } else {
        (0, Popup::None)
    };

    let popup_len = if popup_line_count > 0 {
        popup_line_count + 1
    } else {
        0
    } as u16;

    let layout = Layout::new(
        Direction::Vertical,
        [
            Constraint::Min(1),
            Constraint::Length(popup_len),
            Constraint::Length(if state.prompt.data.is_some() { 2 } else { 0 }),
        ],
    )
    .split(frame.size());

    frame.render_widget(state.screen(), layout[0]);

    match popup {
        Popup::None => (),
        Popup::Paragraph(paragraph) => frame.render_widget(paragraph, layout[1]),
        Popup::Table(table) => frame.render_widget(table, layout[1]),
    }

    if let Some(prompt_data) = state.prompt.data.take() {
        let prompt = TextPrompt::new(prompt_data.prompt_text.clone()).with_block(popup_block());
        frame.render_stateful_widget(prompt, layout[2], &mut state.prompt.state);
        let (cx, cy) = state.prompt.state.cursor();
        frame.set_cursor(cx, cy);
        state.prompt.data = Some(prompt_data);
    }
}

fn format_command<'a>(config: &Config, cmd: &'a CmdMetaBuffer) -> Text<'a> {
    [Line::styled(
        format!(
            "$ {}{}",
            cmd.args,
            if cmd.out.is_some() { "" } else { "..." }
        ),
        &config.style.command,
    )]
    .into_iter()
    .chain(cmd.out.iter().flat_map(|out| Text::raw(out).lines))
    .collect::<Vec<Line>>()
    .into()
}

fn format_keybinds_menu<'b>(
    config: &Config,
    pending: &'b SubmenuOp,
    item: &'b Item,
) -> (usize, Popup<'b>) {
    let style = &config.style;

    let non_target_binds = keybinds::list(pending)
        .filter(|keybind| !keybind.op.is_target_op())
        .collect::<Vec<_>>();

    let mut pending_binds_column = vec![];
    pending_binds_column.push(Line::styled(format!("{}", pending), &style.command));
    for (op, binds) in non_target_binds
        .iter()
        .group_by(|bind| bind.op)
        .into_iter()
        .filter(|(op, _binds)| !matches!(op, Op::Submenu(_)))
    {
        pending_binds_column.push(Line::from(vec![
            Span::styled(
                binds
                    .into_iter()
                    .map(|bind| Keybind::format_key(bind))
                    .join(" "),
                &style.hotkey,
            ),
            Span::styled(format!(" {}", op), Style::new()),
        ]));
    }

    let submenus = non_target_binds
        .iter()
        .filter(|bind| matches!(bind.op, Op::Submenu(_)))
        .collect::<Vec<_>>();

    let mut submenu_binds_column = vec![];
    if !submenus.is_empty() {
        submenu_binds_column.push(Line::styled("Submenu", &style.command));
    }
    for bind in submenus {
        let Op::Submenu(submenu) = bind.op else {
            unreachable!();
        };

        submenu_binds_column.push(Line::from(vec![
            Span::styled(Keybind::format_key(bind), &style.hotkey),
            Span::styled(format!(" {}", submenu), Style::new()),
        ]));
    }

    let mut target_binds_column = vec![];
    if let Some(target_data) = &item.target_data {
        let target_binds = keybinds::list(pending)
            .filter(|keybind| keybind.op.is_target_op())
            .filter(|keybind| OpTrait::get_action(&keybind.op, Some(target_data)).is_some())
            .collect::<Vec<_>>();

        if !target_binds.is_empty() {
            target_binds_column.push(item.display.clone());
        }

        for bind in target_binds {
            target_binds_column.push(Line::from(vec![
                Span::styled(Keybind::format_key(bind), &style.hotkey),
                Span::styled(format!(" {}", bind.op), Style::new()),
            ]));
        }
    }

    let rows = pending_binds_column
        .into_iter()
        .zip_longest(submenu_binds_column)
        .zip_longest(target_binds_column)
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
        Constraint::Length(25),
    ];
    (
        rows.len(),
        Popup::Table(Table::new(rows, widths).block(popup_block())),
    )
}

fn command_popup(text: Text<'_>) -> Popup {
    Popup::Paragraph(Paragraph::new(text).block(popup_block()))
}

fn popup_block() -> Block<'static> {
    Block::new()
        .borders(Borders::TOP)
        .border_style(Style::new().dim())
        .border_type(ratatui::widgets::BorderType::Plain)
}
