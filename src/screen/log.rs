use super::Screen;
use crate::{config::Config, items::log, Res};
use git2::Repository;
use ratatui::prelude::Rect;
use std::rc::Rc;

pub(crate) fn create(
    config: Rc<Config>,
    repo: Rc<Repository>,
    size: Rect,
    reference: Option<String>,
) -> Res<Screen> {
    Screen::new(
        Rc::clone(&config),
        size,
        Box::new(move || log(&config, &repo, usize::MAX, reference.clone())),
    )
}
