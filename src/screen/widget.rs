use crate::theme;

use super::Screen;
use ratatui::prelude::*;
use ratatui::widgets::Widget;

impl Widget for &Screen {
    fn render(self, area: Rect, buf: &mut Buffer) {
        main_ui_lines(self)
            .skip(self.scroll as usize)
            .take(area.height as usize)
            .enumerate()
            .for_each(|(i, line)| {
                buf.set_line(0, i as u16, &line, area.width);
            });
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

            Some((i, item, highlight_depth.is_some()))
        })
        .flat_map(|(i, item, should_highlight)| {
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

            if should_highlight {
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
