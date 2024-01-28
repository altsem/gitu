use crate::{
    git,
    items::{self, Item},
};

pub(crate) fn create(args: &[&str]) -> Vec<Item> {
    let diff = git::diff(args);
    items::create_diff_items(&diff, &0).collect()
}
