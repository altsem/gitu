use crate::{
    git,
    items::{self},
    util,
};

use super::Screen;

pub(crate) fn create(size: (u16, u16), args: &[String]) -> Screen {
    let str_args = util::str_vec(args);
    let log = git::log(&str_args);

    Screen::new(
        size,
        Box::new(move || items::create_log_items(&log).collect()),
    )
}
