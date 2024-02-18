use crate::{
    git,
    items::{self, Item},
    util, Config, Res,
};
use ansi_to_tui::IntoText;
use ratatui::{prelude::Rect, text::Text};

use super::Screen;

pub(crate) fn create(config: &Config, size: Rect, reference: String) -> Res<Screen> {
    let config = config.clone();

    Screen::new(
        size,
        Box::new(move || {
            let summary = git::show_summary(&config.dir, &reference)?;
            let show = git::show(&config.dir.clone(), &reference)?;

            let commit_text = summary.replace("[m", "[0m").into_text()?;

            Ok([
                Item {
                    display: Text::from(commit_text.lines[0].clone()),
                    ..Default::default()
                },
                Item {
                    display: Text::from(commit_text.lines[1..].to_vec()),
                    unselectable: true,
                    ..Default::default()
                },
            ]
            .into_iter()
            .chain(items::create_diff_items(&show, &0))
            .collect())
        }),
    )
}
