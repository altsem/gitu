use crate::config::Config;
use crate::items::Item;
use crate::keybinds;
use crate::keybinds::Keybind;
use crate::keybinds::Op;
use crate::keybinds::SubmenuOp;
use crate::list_target_ops;
use crate::CmdMeta;
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
use tui_prompts::State as _;
use tui_prompts::TextPrompt;

enum Popup<'a> {
    None,
    Paragraph(Paragraph<'a>),
    Table(Table<'a>),
}

pub(crate) fn ui<B: Backend>(frame: &mut Frame, state: &mut State) {
    let (popup_line_count, popup): (usize, Popup) = if let Some(ref cmd) = state.cmd_meta {
        let lines = format_command(&state.config, cmd);
        (lines.len(), command_popup(lines))
    } else if state.pending_submenu_op != SubmenuOp::None {
        format_keybinds_menu::<B>(
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
            Constraint::Length(if state.prompt.is_some() { 1 } else { 0 }),
        ],
    )
    .split(frame.size());

    frame.render_widget(state.screen(), layout[0]);

    match popup {
        Popup::None => (),
        Popup::Paragraph(paragraph) => frame.render_widget(paragraph, layout[1]),
        Popup::Table(table) => frame.render_widget(table, layout[1]),
    }

    if let Some(prompt) = state.prompt {
        frame.render_stateful_widget(
            TextPrompt::new(prompt_text(prompt)),
            layout[2],
            &mut state.prompt_state,
        );
        let (cx, cy) = state.prompt_state.cursor();
        frame.set_cursor(cx, cy);
    }
}

fn prompt_text(prompt: Op) -> std::borrow::Cow<'static, str> {
    match prompt {
        Op::CheckoutNewBranch => "New branch name: ",
        _ => unimplemented!(),
    }
    .into()
}

fn format_command<'a>(config: &Config, cmd: &'a CmdMeta) -> Vec<Line<'a>> {
    [Line::styled(
        format!(
            "$ {}{}",
            cmd.args,
            if cmd.out.is_some() { "" } else { "..." }
        ),
        &config.color.command,
    )]
    .into_iter()
    .chain(cmd.out.iter().flat_map(|out| Text::raw(out).lines))
    .collect::<Vec<Line>>()
}

fn format_keybinds_menu<'b, B: Backend>(
    config: &Config,
    pending: &'b SubmenuOp,
    item: &'b Item,
) -> (usize, Popup<'b>) {
    let color = &config.color;

    let non_target_binds = keybinds::list(pending)
        .filter(|keybind| !matches!(keybind.op, keybinds::Op::Target(_)))
        .collect::<Vec<_>>();

    let mut pending_binds_column = vec![];
    pending_binds_column.push(Line::styled(format!("{:?}", pending), &color.command));
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
                &color.hotkey,
            ),
            Span::styled(format!(" {:?}", op), Style::new()),
        ]));
    }

    let submenus = non_target_binds
        .iter()
        .filter(|bind| matches!(bind.op, Op::Submenu(_)))
        .collect::<Vec<_>>();

    let mut submenu_binds_column = vec![];
    if !submenus.is_empty() {
        submenu_binds_column.push(Line::styled("Submenu", &color.command));
    }
    for bind in submenus {
        let Op::Submenu(submenu) = bind.op else {
            unreachable!();
        };

        submenu_binds_column.push(Line::from(vec![
            Span::styled(Keybind::format_key(bind), &color.hotkey),
            Span::styled(format!(" {:?}", submenu), Style::new()),
        ]));
    }

    let mut target_binds_column = vec![];
    if let Some(target_data) = &item.target_data {
        let target_ops = list_target_ops::<B>(target_data).collect::<Vec<_>>();
        let target_binds = keybinds::list(pending)
            .filter(|keybind| matches!(keybind.op, keybinds::Op::Target(_)))
            .filter(|keybind| {
                let Op::Target(target) = keybind.op else {
                    unreachable!();
                };

                target_ops.contains(&target)
            })
            .collect::<Vec<_>>();

        if !target_binds.is_empty() {
            target_binds_column.push(item.display.clone());
        }

        for bind in target_binds {
            let Op::Target(target) = bind.op else {
                unreachable!();
            };

            target_binds_column.push(Line::from(vec![
                Span::styled(Keybind::format_key(bind), &color.hotkey),
                Span::styled(format!(" {:?}", target), Style::new()),
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
        Constraint::Length(25),
        Constraint::Length(12),
        Constraint::Length(25),
    ];
    (
        rows.len(),
        Popup::Table(
            Table::new(rows, widths).block(
                Block::new()
                    .borders(Borders::TOP)
                    .border_style(Style::new().dim())
                    .border_type(ratatui::widgets::BorderType::Plain),
            ),
        ),
    )
}

fn command_popup(lines: Vec<Line>) -> Popup {
    Popup::Paragraph(
        Paragraph::new(lines).block(
            Block::new()
                .borders(Borders::TOP)
                .border_style(Style::new().dim())
                .border_type(ratatui::widgets::BorderType::Plain),
        ),
    )
}
