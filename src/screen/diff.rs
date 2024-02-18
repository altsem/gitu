use std::rc::Rc;

use git2::Repository;
use ratatui::prelude::Rect;

use super::Screen;
use crate::{git, items, util, Res};

pub(crate) fn create(repo: Rc<Repository>, size: Rect, args: Vec<String>) -> Res<Screen> {
    Screen::new(
        size,
        Box::new(move || {
            let str_args = util::str_vec(&args);
            let diff = git::diff(repo.as_ref(), &str_args)?;

            Ok(items::create_diff_items(&diff, &0).collect())
        }),
    )
}
