use super::Screen;
use crate::{git, items};
use std::collections::HashSet;

pub(crate) fn create(size: (u16, u16), args: Vec<String>) -> Screen {
    let args_clone = args.clone();
    let args = args.iter().map(String::as_str).collect::<Vec<_>>();

    Screen {
        cursor: 0,
        scroll: 0,
        size,
        refresh_items: Box::new(move || {
            items::create_log_items(git::log(
                &args_clone.iter().map(String::as_str).collect::<Vec<_>>(),
            ))
            .collect()
        }),
        items: items::create_log_items(git::log(&args)).collect(),
        collapsed: HashSet::new(),
        command: None,
    }
}
