use std::{iter, rc::Rc};

use super::Screen;
use crate::{
    config::{Config, StyleConfigEntry},
    items::{Item, TargetData},
    Res,
};
use git2::{Reference, Repository};
use ratatui::{
    prelude::Rect,
    text::{Line, Span},
};

pub(crate) fn create(config: Rc<Config>, repo: Rc<Repository>, size: Rect) -> Res<Screen> {
    Screen::new(
        Rc::clone(&config),
        size,
        Box::new(move || {
            let style = &config.style;

            Ok(iter::once(Item {
                id: "local_branches".into(),
                display: Line::styled("Branches".to_string(), &style.section_header),
                section: true,
                depth: 0,
                ..Default::default()
            })
            .chain(create_references_section_items(
                &repo,
                Reference::is_branch,
                &style.branch,
            )?)
            .chain(iter::once(Item {
                id: "remote_branches".into(),
                display: Line::styled("Remote".to_string(), &style.section_header),
                section: true,
                depth: 0,
                ..Default::default()
            }))
            .chain(create_references_section_items(
                &repo,
                Reference::is_remote,
                &style.remote,
            )?)
            .chain(iter::once(Item {
                id: "tags".into(),
                display: Line::styled("Tags".to_string(), &style.section_header),
                section: true,
                depth: 0,
                ..Default::default()
            }))
            .chain(create_references_section_items(
                &repo,
                Reference::is_tag,
                &style.tag,
            )?)
            .collect())
        }),
    )
}

fn create_references_section_items<'a, F>(
    repo: &'a Repository,
    filter: F,
    style: &'a StyleConfigEntry,
) -> Res<impl Iterator<Item = Item> + 'a>
where
    F: FnMut(&Reference<'a>) -> bool + 'a,
{
    Ok(repo
        .references()?
        .filter_map(Result::ok)
        .filter(filter)
        .map(move |reference| {
            let name = Span::styled(reference.shorthand().unwrap().to_string(), style);
            let head = repo.head().ok();

            let prefix = Span::raw(
                if reference.target() == head.as_ref().and_then(Reference::target) {
                    "* "
                } else {
                    "  "
                },
            );

            Item {
                id: name.clone().content,
                display: Line::from(vec![prefix, name.clone()]),
                depth: 1,
                target_data: Some(TargetData::Branch(name.content.into())),
                ..Default::default()
            }
        }))
}
