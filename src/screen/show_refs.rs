use std::{
    collections::{hash_map::Entry, HashMap},
    iter,
    rc::Rc,
};

use super::Screen;
use crate::{
    config::{Config, StyleConfigEntry},
    items::{self, Item, TargetData},
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
            .chain(create_references_section(
                &repo,
                Reference::is_branch,
                &style.branch,
            )?)
            .chain(create_remotes_sections(
                &repo,
                &style.section_header,
                &style.remote,
            )?)
            .chain(create_tags_section(
                &repo,
                &style.section_header,
                &style.tag,
            )?)
            .collect())
        }),
    )
}

fn create_remotes_sections<'a>(
    repo: &'a Repository,
    header_style: &'a StyleConfigEntry,
    item_style: &'a StyleConfigEntry,
) -> Res<impl Iterator<Item = Item> + 'a> {
    let all_remotes = create_references_section(repo, Reference::is_remote, item_style)?;
    let mut remotes = HashMap::new();
    for remote in all_remotes {
        let name = match remote.id.split_once('/') {
            None => remote.id.as_ref(),
            Some((name, _)) => name,
        };

        match remotes.entry(name.to_string()) {
            Entry::Vacant(entry) => {
                entry.insert(vec![remote]);
            }
            Entry::Occupied(mut entry) => {
                entry.get_mut().push(remote);
            }
        }
    }

    Ok(remotes.into_iter().flat_map(move |(name, items)| {
        let header = format!("Remote {name}");
        vec![
            items::blank_line(),
            Item {
                id: name.into(),
                display: Line::styled(header, header_style),
                section: true,
                depth: 0,
                ..Default::default()
            },
        ]
        .into_iter()
        .chain(items)
    }))
}

fn create_tags_section<'a>(
    repo: &'a Repository,
    header_style: &'a StyleConfigEntry,
    item_style: &'a StyleConfigEntry,
) -> Res<impl Iterator<Item = Item> + 'a> {
    let mut tags = create_references_section(repo, Reference::is_tag, item_style)?;
    Ok(match tags.next() {
        Some(item) => vec![
            items::blank_line(),
            Item {
                id: "tags".into(),
                display: Line::styled("Tags".to_string(), header_style),
                section: true,
                depth: 0,
                ..Default::default()
            },
            item,
        ],
        None => vec![],
    }
    .into_iter()
    .chain(tags))
}

fn create_references_section<'a, F>(
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

            Item {
                id: name.clone().content,
                display: Line::from(vec![create_prefix(repo, &reference), name.clone()]),
                depth: 1,
                target_data: Some(TargetData::Branch(name.content.into())),
                ..Default::default()
            }
        }))
}

fn create_prefix(repo: &Repository, reference: &Reference) -> Span<'static> {
    let head = repo.head().ok();

    Span::raw(if repo.head_detached().unwrap_or(false) {
        if reference.target() == head.as_ref().and_then(Reference::target) {
            "? "
        } else {
            "  "
        }
    } else if reference.name() == head.as_ref().and_then(Reference::name) {
        "* "
    } else {
        "  "
    })
}
