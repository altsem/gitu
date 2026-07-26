use super::Screen;
use crate::{Res, config::Config, items::log};
use git2::{Oid, Repository};
use regex::Regex;
use std::{rc::Rc, sync::Arc};

pub(crate) fn create(
    config: Arc<Config>,
    repo: Rc<Repository>,
    size: (u16, u16),
    limit: usize,
    rev: Option<Oid>,
    msg_regex: Option<Regex>,
) -> Res<Screen> {
    Screen::new(
        Arc::clone(&config),
        size,
        Box::new(move || log(&repo, limit, rev, msg_regex.clone())),
    )
}
