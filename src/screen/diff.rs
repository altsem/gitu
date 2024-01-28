use crate::{git, items, util};

use super::Screen;

pub(crate) fn create(args: Vec<String>) -> Screen {
    Screen::new(Box::new(move || {
        let str_args = util::str_vec(&args);
        let diff = git::diff(&str_args);

        items::create_diff_items(&diff, &0).collect()
    }))
}
