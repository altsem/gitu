use super::Screen;
use crate::{config::Config, items::log, Res};
use git2::{Oid, Repository};
use ratatui::prelude::Rect;
use regex::Regex;
use std::rc::Rc;

pub(crate) fn create(
    config: Rc<Config>,
    repo: Rc<Repository>,
    size: Rect,
    limit: usize,
    rev: Option<Oid>,
    msg_regex: Option<Regex>,
) -> Res<Screen> {
    Screen::new(
        Rc::clone(&config),
        size,
        Box::new(move || log(&config, &repo, limit, rev, &msg_regex)),
    )
}
