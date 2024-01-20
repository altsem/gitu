use crate::screen::Screen;
use crate::theme;
use ratatui::prelude::*;
use ratatui::text::Span;
use ratatui::text::Text;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Paragraph;
use ratatui::widgets::StatefulWidget;
use ratatui::Frame;

pub(crate) fn ui(frame: &mut Frame, screen: &mut Screen) {
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

    frame.render_stateful_widget(ScreenWidget, layout[0], screen);

    if !cmd_text.is_empty() {
        frame.render_widget(command_popup(cmd_text), layout[1]);
    }
}

struct ScreenWidget;

impl StatefulWidget for ScreenWidget {
    type State = Screen;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        for (i, line) in main_ui_lines(&*state)
            .into_iter()
            .skip(state.scroll as usize)
            .take(area.height as usize)
            .enumerate()
        {
            buf.set_line(0, i as u16, &line, area.width);
        }
    }
}

fn main_ui_lines(screen: &Screen) -> impl Iterator<Item = Line> {
    screen
        .collapsed_items_iter()
        .scan(None, |highlight_depth, (i, item)| {
            if screen.cursor == i {
                *highlight_depth = Some(item.depth);
            } else if highlight_depth.is_some_and(|s| s >= item.depth) {
                *highlight_depth = None;
            };

            Some((i, item, *highlight_depth))
        })
        .flat_map(|(i, item, highlight_depth)| {
            let mut text = if let Some((ref text, style)) = item.display {
                use ansi_to_tui::IntoText;
                let mut text = text.into_text().expect("Couldn't read ansi codes");
                text.patch_style(style);
                text
            } else {
                Text::raw("")
            };

            if screen.is_collapsed(item) {
                text.lines
                    .last_mut()
                    .expect("No last line found")
                    .spans
                    .push("…".into());
            }

            for line in text.lines.iter_mut() {
                let padding = (screen.size.0 as usize).saturating_sub(line.width());

                line.spans.push(Span::styled(
                    " ".repeat(padding),
                    line.spans.first().unwrap().style,
                ));
            }

            if highlight_depth.is_some() {
                highlight_section(&mut text, screen, i);
            }

            text
        })
}

fn highlight_section(text: &mut Text<'_>, screen: &Screen, i: usize) {
    for line in &mut text.lines {
        for span in &mut line.spans {
            if span.style.bg.is_none() {
                span.style.bg = Some(if screen.cursor == i {
                    theme::CURRENT_THEME.highlight
                } else {
                    theme::CURRENT_THEME.dim_highlight
                })
            }
        }
    }
}

fn format_command<'a, 'b>(cmd: &'a crate::command::IssuedCommand) -> Vec<Line<'b>> {
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
