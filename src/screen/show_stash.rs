use std::{iter, rc::Rc, sync::Arc};

use crate::{
    Res,
    config::Config,
    git,
    item_data::{ItemData, SectionHeader},
    items::{self, Item, hash},
};
use git2::Repository;
use ratatui::layout::Size;

use super::Screen;

pub(crate) fn create(
    config: Arc<Config>,
    repo: Rc<Repository>,
    size: Size,
    stash_ref: String,
) -> Res<Screen> {
    Screen::new(
        Arc::clone(&config),
        size,
        Box::new(move || {
            let commit = git::show_summary(repo.as_ref(), &stash_ref)?;
            let show = git::stash_show(repo.as_ref(), &stash_ref)?;
            let details = commit.details.lines();

            Ok(iter::once(Item {
                id: hash(["stash_section", &commit.hash]),
                depth: 0,
                data: ItemData::Header(SectionHeader::Commit(commit.hash.clone())),
                ..Default::default()
            })
            .chain(details.into_iter().map(|line| Item {
                id: hash(["stash", &commit.hash]),
                depth: 1,
                unselectable: true,
                data: ItemData::Raw(line.to_string()),
                ..Default::default()
            }))
            .chain([items::blank_line()])
            .chain(items::create_diff_items(&Rc::new(show), 0, false))
            .collect())
        }),
    )
}
