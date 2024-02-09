use ratatui::prelude::Rect;

use super::Screen;
use crate::{git, items, util, Config, Res};

pub(crate) fn create(config: &Config, size: Rect, args: Vec<String>) -> Res<Screen> {
    let config = config.clone();
    Screen::new(
        size,
        Box::new(move || {
            let str_args = util::str_vec(&args);
            let diff = git::diff(&config.dir, &str_args)?;

            Ok(items::create_diff_items(&diff, &0).collect())
        }),
    )
}
