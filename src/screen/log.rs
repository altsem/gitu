use crate::{
    git,
    items::{self},
    util, Res,
};

use super::Screen;

pub(crate) fn create(args: Vec<String>) -> Res<Screen> {
    Screen::new(Box::new(move || {
        let str_args = util::str_vec(&args);
        let log = git::log(&str_args)?;

        Ok(items::create_log_items(&log).collect())
    }))
}
