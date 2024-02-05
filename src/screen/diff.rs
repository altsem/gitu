use crate::{git, items, util, Res};

use super::Screen;

pub(crate) fn create(args: Vec<String>) -> Res<Screen> {
    Screen::new(Box::new(move || {
        let str_args = util::str_vec(&args);
        let diff = git::diff(&str_args)?;

        Ok(items::create_diff_items(&diff, &0).collect())
    }))
}
