use crate::{
    git,
    items::{self},
    util,
};

use super::Screen;

pub(crate) fn create(args: Vec<String>) -> Screen {
    Screen::new(Box::new(move || {
        let str_args = util::str_vec(&args);
        let log = git::log(&str_args);

        items::create_log_items(&log).collect()
    }))
}
