use ratatui::prelude::Rect;

use super::Screen;
use crate::{
    git,
    items::{self},
    util, Config, Res,
};

pub(crate) fn create(config: &Config, size: Rect, args: Vec<String>) -> Res<Screen> {
    let path_buf = config.dir.clone();
    Screen::new(
        size,
        Box::new(move || {
            let str_args = util::str_vec(&args);
            // TODO Replace with libgit2
            let log = git::log(&path_buf, &str_args)?;

            Ok(items::create_log_items(&log).collect())
        }),
    )
}
