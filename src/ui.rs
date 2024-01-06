use super::collapsed_items_iter;
use super::Item;
use super::State;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::text::Text;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub(crate) fn ui(frame: &mut Frame, state: &State) {
    let mut highlight_depth = None;

    let mut lines = collapsed_items_iter(&state.collapsed, &state.items)
        .flat_map(|(i, item)| {
            let mut text = if let Some(ref text) = item.header {
                Text::styled(text, Style::new().fg(Color::Yellow))
            } else if let Item {
                line: Some(line), ..
            } = item
            {
                use ansi_to_tui::IntoText;
                line.into_text().expect("Couldn't read ansi codes")
            } else {
                panic!("Couldn't format item");
            };

            if state.collapsed.contains(item) {
                text.lines
                    .last_mut()
                    .expect("No last line found")
                    .spans
                    .push("…".into());
            }

            if state.selected == i {
                highlight_depth = Some(item.depth);
            } else if highlight_depth.is_some_and(|hd| hd >= item.depth) {
                highlight_depth = None;
            }

            text.patch_style(if highlight_depth.is_some() {
                Style::new()
            } else {
                Style::new().add_modifier(Modifier::DIM)
            });

            text
        })
        .collect::<Vec<_>>();

    if let Some(ref cmd) = state.command {
        lines.extend(Text::from("\n".to_string() + &cmd.args.clone()).lines);
        lines.extend(
            Text::raw(
                String::from_utf8(cmd.output.clone())
                    .expect("Error turning command output to String"),
            )
            .lines,
        );
    }

    frame.render_widget(Paragraph::new(lines), frame.size());
}
