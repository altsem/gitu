use crate::{
    git,
    items::{self, Item},
};

pub(crate) fn create(args: &[&str]) -> Vec<Item> {
    let log = git::log(args);
    items::create_log_items(&log).collect()
}
