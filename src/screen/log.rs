use super::Screen;
use crate::{items::log, Res};
use git2::Repository;
use ratatui::prelude::Rect;
use std::rc::Rc;

pub(crate) fn create(repo: Rc<Repository>, size: Rect) -> Res<Screen> {
    Screen::new(size, Box::new(move || log(&repo, usize::MAX)))
}
