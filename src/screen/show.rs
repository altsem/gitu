use crate::{
    git,
    items::{self, Item},
    util, Config, Res,
};
use ansi_to_tui::IntoText;
use std::iter;

use super::Screen;

pub(crate) fn create(config: &Config, args: Vec<String>) -> Res<Screen> {
    let config = config.clone();
    Screen::new(Box::new(move || {
        let str_args = util::str_vec(&args);
        let summary = git::show_summary(&config.dir, &str_args)?;
        let show = git::show(&config.dir.clone(), &str_args)?;

        Ok(iter::once(Item {
            display: summary.into_text()?,
            ..Default::default()
        })
        .chain(items::create_diff_items(&config, &show, &0))
        .collect())
    }))
}
