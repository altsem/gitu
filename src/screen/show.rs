use std::{iter, rc::Rc};

use crate::{
    config::Config,
    git,
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
                ..Default::default()
            })
            .chain(details.into_iter().map(|_| Item {
                id: hash(["commit", &commit.hash]),
                depth: 1,
                unselectable: true,
                ..Default::default()
            }))
            .chain([items::blank_line()])
            .chain(items::create_diff_items(&Rc::new(show), 0, false))
            .collect())
        }),
    )
}
