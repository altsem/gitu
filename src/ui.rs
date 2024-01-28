use crate::items::Item;
use crate::keybinds;
use crate::keybinds::Keybind;
use crate::keybinds::Op;
use crate::keybinds::TransientOp;
use crate::list_target_ops;
use crate::theme::CURRENT_THEME;
use crate::State;
use itertools::EitherOrBoth;
use itertools::Itertools;
use ratatui::prelude::*;
use ratatui::text::Text;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Row;
use ratatui::widgets::Table;
use ratatui::Frame;

enum Popup<'a> {
    None,
    Paragraph(Paragraph<'a>),
    Table(Table<'a>),
}

pub(crate) fn ui(frame: &mut Frame, state: &State) {
    let (popup_line_count, popup): (usize, Popup) =
        if state.pending_transient_op != TransientOp::None {
            format_keybinds_menu(
                &state.pending_transient_op,
                state.screen().get_selected_item(),
            )
        } else if let Some(ref cmd) = state.command {
            let lines = format_command(cmd);
            (lines.len(), command_popup(lines))
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
        [Constraint::Min(1), Constraint::Length(popup_len)],
    )
    .split(frame.size());

    frame.render_widget(state.screen(), layout[0]);

    match popup {
        Popup::None => (),
        Popup::Paragraph(paragraph) => frame.render_widget(paragraph, layout[1]),
        Popup::Table(table) => frame.render_widget(table, layout[1]),
    }
}

fn format_command<'b>(cmd: &crate::command::IssuedCommand) -> Vec<Line<'b>> {
    Text::styled(
        format!(
            "$ {}{}",
            cmd.args,
            if cmd.finish_acked { "" } else { "..." }
        ),
        Style::new().fg(CURRENT_THEME.command),
    )
    .lines
    .into_iter()
    .chain(
        Text::raw(
            String::from_utf8(cmd.output.clone()).expect("Error turning command output to String"),
        )
        .lines,
    )
    .collect::<Vec<Line>>()
}

fn format_keybinds_menu<'b>(pending: &'b TransientOp, item: &'b Item) -> (usize, Popup<'b>) {
    let non_target_binds = keybinds::list(pending)
        .filter(|keybind| !matches!(keybind.op, keybinds::Op::Target(_)))
        .collect::<Vec<_>>();

    let mut pending_binds_column = vec![];
    pending_binds_column.push(Line::styled(
        format!("{:?}", pending),
        Style::new().fg(CURRENT_THEME.command),
    ));
    for bind in non_target_binds
        .iter()
        .filter(|bind| !matches!(bind.op, Op::Transient(_)))
    {
        pending_binds_column.push(Line::from(vec![
            Span::styled(
                Keybind::format_key(bind),
                Style::new().fg(CURRENT_THEME.command),
            ),
            Span::styled(format!(" {:?}", bind.op), Style::new()),
        ]));
    }

    let transients = non_target_binds
        .iter()
        .filter(|bind| matches!(bind.op, Op::Transient(_)))
        .collect::<Vec<_>>();

    let mut transient_binds_column = vec![];
    if !transients.is_empty() {
        transient_binds_column.push(Line::styled(
            "Transient",
            Style::new().fg(CURRENT_THEME.command),
        ));
    }
    for bind in transients {
        let Op::Transient(transient) = bind.op else {
            unreachable!();
        };

        transient_binds_column.push(Line::from(vec![
            Span::styled(
                Keybind::format_key(bind),
                Style::new().fg(CURRENT_THEME.command),
            ),
            Span::styled(format!(" {:?}", transient), Style::new()),
        ]));
    }

    let mut target_binds_column = vec![];
    if let Some(target_data) = &item.target_data {
        let target_ops = list_target_ops(target_data).collect::<Vec<_>>();
        let target_binds = keybinds::list(pending)
            .filter(|keybind| matches!(keybind.op, keybinds::Op::Target(_)))
            .collect::<Vec<_>>();

        if !target_binds.is_empty() {
            target_binds_column.extend(item.display.lines.clone());
        }

        for bind in target_binds {
            let Op::Target(target) = bind.op else {
                unreachable!();
            };

            if !target_ops.contains(&&target) {
                continue;
            }

            target_binds_column.push(Line::from(vec![
                Span::styled(
                    Keybind::format_key(bind),
                    Style::new().fg(CURRENT_THEME.command),
                ),
                Span::styled(format!(" {:?}", target), Style::new()),
            ]));
        }
    }

    let rows = pending_binds_column
        .into_iter()
        .zip_longest(transient_binds_column)
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
        Constraint::Length(20),
        Constraint::Length(20),
        Constraint::Length(20),
    ];
    (rows.len(), Popup::Table(Table::new(rows, widths)))
}

fn command_popup(lines: Vec<Line>) -> Popup {
    Popup::Paragraph(
        Paragraph::new(lines).block(
            Block::new()
                .borders(Borders::TOP)
                .border_style(Style::new().fg(CURRENT_THEME.highlight))
                .border_type(ratatui::widgets::BorderType::Plain),
        ),
    )
}
