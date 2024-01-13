use super::Screen;
use crate::{git, items};
use std::collections::HashSet;

pub(crate) fn create(size: (u16, u16)) -> Screen {
    Screen {
        cursor: 0,
        scroll: 0,
        size,
        refresh_items: Box::new(move || items::create_log_items(git::log()).collect()),
        items: items::create_log_items(git::log()).collect(),
        collapsed: HashSet::new(),
        command: None,
    }
}
