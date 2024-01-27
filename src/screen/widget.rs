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
        .map(|(i, item)| (i, item, get_display_text(&item)))
        .flat_map(|(i, item, text)| text.lines.into_iter().map(move |line| (i, item, line)))
        .scan(None, |highlight_depth, (i, item, mut line)| {
            if screen.is_collapsed(&item) {
                if line.width() > 0 {
                    line.spans.push("…".into());
                }
            }

            extend_bg_to_line_end(&mut line, screen);
            if screen.cursor == i {
                *highlight_depth = Some(item.depth);
            } else if highlight_depth.is_some_and(|s| s >= item.depth) {
                *highlight_depth = None;
            };

            if highlight_depth.is_some() {
                highlight_line(&mut line, screen, i);
            }

            Some((i, item, line))
        })
        .map(|(_i, _item, line)| line)
}

fn get_display_text<'a>(item: &crate::items::Item) -> Text<'a> {
    let (ref text, style) = item.display;
    use ansi_to_tui::IntoText;
    let mut text = text.into_text().expect("Couldn't read ansi codes");
    text.patch_style(style);
    text
}

fn extend_bg_to_line_end(line: &mut Line<'_>, screen: &Screen) {
    let padding = (screen.size.0 as usize).saturating_sub(line.width());

    line.spans.push(Span::styled(
        " ".repeat(padding),
        line.spans.first().unwrap().style,
    ));
}

fn highlight_line(line: &mut Line<'_>, screen: &Screen, i: usize) {
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
