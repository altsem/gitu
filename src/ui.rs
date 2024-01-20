use crate::screen::Screen;
use crate::theme;
use ratatui::prelude::*;
use ratatui::text::Text;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub(crate) fn ui(frame: &mut Frame, screen: &Screen) {
    let cmd_text = if let Some(ref cmd) = screen.command {
        format_command(cmd)
    } else {
        vec![]
    };

    let cmd_len = cmd_text.len() as u16;
    let layout = Layout::new(
        Direction::Vertical,
        [
            Constraint::Min(1),
            Constraint::Length(if cmd_len > 0 { cmd_len + 1 } else { 0 }),
        ],
    )
    .split(frame.size());

    frame.render_widget(screen, layout[0]);

    if !cmd_text.is_empty() {
        frame.render_widget(command_popup(cmd_text), layout[1]);
    }
}

fn format_command<'b>(cmd: &crate::command::IssuedCommand) -> Vec<Line<'b>> {
    Text::styled(
        format!(
            "$ {}{}",
            cmd.args,
            if cmd.finish_acked { "" } else { "..." }
        ),
        Style::new().fg(theme::CURRENT_THEME.command),
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

fn command_popup(output_lines: Vec<Line>) -> Paragraph {
    Paragraph::new(output_lines).block(
        Block::new()
            .borders(Borders::TOP)
            .border_style(Style::new().fg(theme::CURRENT_THEME.highlight))
            .border_type(ratatui::widgets::BorderType::Plain),
    )
}
