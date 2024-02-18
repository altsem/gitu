use std::rc::Rc;

use crate::{
    git,
    items::{self, Item},
    theme::CURRENT_THEME,
    Res,
};
use git2::Repository;
use ratatui::{
    prelude::Rect,
    style::Stylize,
    text::{Line, Text},
};

use super::Screen;

pub(crate) fn create(repo: Rc<Repository>, size: Rect, reference: String) -> Res<Screen> {
    Screen::new(
        size,
        Box::new(move || {
            let commit = git::show_summary(repo.as_ref(), &reference)?;
            let show = git::show(repo.as_ref(), &reference)?;
            let mut details = Text::from(commit.details);
            details.lines.push(Line::raw(""));

            Ok(vec![
                Item {
                    id: format!("commit_section_{}", commit.hash).into(),
                    display: Text::from(
                        format!("commit {}", commit.hash).fg(CURRENT_THEME.section),
                    ),
                    section: true,
                    depth: 0,
                    ..Default::default()
                },
                Item {
                    id: format!("commit_{}", commit.hash).into(),
                    display: details,
                    depth: 1,
                    unselectable: true,
                    ..Default::default()
                },
            ]
            .into_iter()
            .chain(items::create_diff_items(&show, &0))
            .collect())
        }),
    )
}
