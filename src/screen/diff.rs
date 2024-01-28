use crate::{git, items, util};

use super::Screen;

pub(crate) fn create(size: (u16, u16), args: &[String]) -> Screen {
    let str_args = util::str_vec(args);
    let diff = git::diff(&str_args);

    Screen::new(
        size,
        Box::new(move || items::create_diff_items(&diff, &0).collect()),
    )
}
