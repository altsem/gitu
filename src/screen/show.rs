use std::{iter, rc::Rc};

use crate::{
    config::Config,
    git,
    items::{self, Item},
    Res,
};
use git2::Repository;
use ratatui::{
    prelude::Rect,
    text::{Line, Text},
};

use super::Screen;

pub(crate) fn create(
    config: Rc<Config>,
    repo: Rc<Repository>,
    size: Rect,
    reference: String,
) -> Res<Screen> {
    Screen::new(
        Rc::clone(&config),
        size,
        Box::new(move || {
            let color = &config.color;
            let commit = git::show_summary(repo.as_ref(), &reference)?;
            let show = git::show(repo.as_ref(), &reference)?;
            let details = Text::from(commit.details).lines;

            Ok(iter::once(Item {
                id: format!("commit_section_{}", commit.hash).into(),
                display: Line::styled(format!("commit {}", commit.hash), &color.section),
                section: true,
                depth: 0,
                ..Default::default()
            })
            .chain(details.into_iter().map(|line| Item {
                id: format!("commit_{}", commit.hash).into(),
                display: line,
                depth: 1,
                unselectable: true,
                ..Default::default()
            }))
            .chain([items::blank_line()])
            .chain(items::create_diff_items(
                Rc::clone(&config),
                &show,
                &0,
                false,
            ))
            .collect())
        }),
    )
}
