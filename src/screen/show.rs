use crate::{
    git,
    items::{self, Item},
    util,
};
use ansi_to_tui::IntoText;
use std::iter;

use super::Screen;

pub(crate) fn create(args: Vec<String>) -> Screen {
    Screen::new(Box::new(move || {
        let str_args = util::str_vec(&args);
        let summary = git::show_summary(&str_args);
        let show = git::show(&str_args);

        iter::once(Item {
            display: summary.into_text().expect("Couldn't read ansi codes"),
            ..Default::default()
        })
        .chain(items::create_diff_items(&show, &0))
        .collect()
    }))
}
