use crate::screen::Screen;
use crate::theme;
use ratatui::text::Span;
use ratatui::text::Text;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub(crate) fn ui(frame: &mut Frame, screen: &Screen) {
    let mut highlight_depth = None;

    let mut lines = screen
        .collapsed_items_iter()
        .flat_map(|(i, item)| {
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

            if screen.cursor == i {
                highlight_depth = Some(item.depth);
            } else if highlight_depth.is_some_and(|hd| hd >= item.depth) {
                highlight_depth = None;
            }

            if highlight_depth.is_some() {
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

            text
        })
        .collect::<Vec<_>>();

    if let Some(ref cmd) = screen.command {
        lines.extend(Text::from("\n".to_string() + &cmd.args.clone()).lines);
        lines.extend(
            Text::raw(
                String::from_utf8(cmd.output.clone())
                    .expect("Error turning command output to String"),
            )
            .lines,
        );
    }

    frame.render_widget(
        Paragraph::new(lines).scroll((screen.scroll, 0)),
        frame.size(),
    );
}
