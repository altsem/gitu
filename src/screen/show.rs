use crate::{
    git,
    items::{self, Item},
    util,
};
use ansi_to_tui::IntoText;
use std::iter;

use super::Screen;

pub(crate) fn create(size: (u16, u16), args: &[String]) -> Screen {
    let str_args = util::str_vec(args);
    let summary = git::show_summary(&str_args);
    let show = git::show(&str_args);

    Screen::new(
        size,
        Box::new(move || {
            iter::once(Item {
                display: summary.into_text().expect("Couldn't read ansi codes"),
                ..Default::default()
            })
            .chain(items::create_diff_items(&show, &0))
            .collect()
        }),
    )
}
