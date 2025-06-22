use std::{iter, rc::Rc};

use crate::{
    config::Config,
    git,
    item_data::{ItemData, SectionHeader},
    items::{self, hash, Item},
    Res,
};
use git2::Repository;
use ratatui::layout::Size;

use super::Screen;

pub(crate) fn create(
    config: Rc<Config>,
    repo: Rc<Repository>,
    size: Size,
    reference: String,
) -> Res<Screen> {
    Screen::new(
        Rc::clone(&config),
        size,
        Box::new(move || {
            let commit = git::show_summary(repo.as_ref(), &reference)?;
            let show = git::show(repo.as_ref(), &reference)?;
            let details = commit.details.lines();

            Ok(iter::once(Item {
                id: hash(["commit_section", &commit.hash]),
                section: true,
                depth: 0,
                data: ItemData::Header(SectionHeader::Commit(commit.hash.clone())),
                ..Default::default()
            })
            .chain(details.into_iter().map(|line| Item {
                id: hash(["commit", &commit.hash]),
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
