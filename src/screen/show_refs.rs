use std::{iter, rc::Rc};

use super::Screen;
use crate::{
    config::{Config, StyleConfigEntry},
    items::{Item, TargetData},
    Res,
};
use git2::{BranchType, Reference, Repository};
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
            .chain(branches(&repo, Some(BranchType::Local), &style.branch)?)
            .chain(iter::once(Item {
                id: "remote_branches".into(),
                display: Line::styled("Remote".to_string(), &style.section_header),
                section: true,
                depth: 0,
                ..Default::default()
            }))
            .chain(branches(&repo, Some(BranchType::Remote), &style.branch)?)
            .chain(iter::once(Item {
                id: "tags".into(),
                display: Line::styled("Tags".to_string(), &style.section_header),
                section: true,
                depth: 0,
                ..Default::default()
            }))
            .chain(
                repo.references()?
                    .filter_map(Result::ok)
                    .filter(Reference::is_tag)
                    .map(|tag| {
                        let name = Span::styled(tag.name().unwrap().to_string(), &style.branch);

                        Item {
                            id: name.clone().content,
                            display: Line::from(vec![prefix(repo.head().ok(), &tag), name.clone()]),
                            depth: 1,
                            target_data: Some(TargetData::Branch(name.content.into())),
                            ..Default::default()
                        }
                    }),
            )
            .collect())
        }),
    )
}

fn branches<'a>(
    repo: &'a Repository,
    filter: Option<BranchType>,
    style: &'a StyleConfigEntry,
) -> Res<impl Iterator<Item = Item> + 'a> {
    Ok(repo
        .branches(filter)?
        .filter_map(Result::ok)
        .map(move |(branch, _branch_type)| {
            let name = Span::styled(branch.name().unwrap().unwrap().to_string(), style);

            Item {
                id: name.clone().content,
                display: Line::from(vec![prefix(repo.head().ok(), branch.get()), name.clone()]),
                depth: 1,
                target_data: Some(TargetData::Branch(name.content.into())),
                ..Default::default()
            }
        }))
}

fn prefix(head: Option<Reference>, target: &Reference) -> Span<'static> {
    Span::raw(
        if target.target() == head.as_ref().and_then(Reference::target) {
            "* "
        } else {
            "  "
        },
    )
}
