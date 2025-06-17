use std::{
    collections::{btree_map::Entry, BTreeMap},
    iter,
    rc::Rc,
};

use super::Screen;
use crate::{
    config::Config,
    error::Error,
    items::{self, hash, Item},
    target_data::{RefKind, SectionHeader, TargetData},
    Res,
};
use git2::{Reference, Repository};
use ratatui::{layout::Size, text::Span};

pub(crate) fn create(config: Rc<Config>, repo: Rc<Repository>, size: Size) -> Res<Screen> {
    Screen::new(
        Rc::clone(&config),
        size,
        Box::new(move || {
            Ok(iter::once(Item {
                id: hash("local_branches"),
                target_data: Some(TargetData::Header(SectionHeader::Branches)),
                section: true,
                depth: 0,
                ..Default::default()
            })
            .chain(create_reference_items(&repo, Reference::is_branch)?.map(|(_, item)| item))
            .chain(create_remotes_sections(&repo)?)
            .chain(create_tags_section(&repo)?)
            .collect())
        }),
    )
}

fn create_remotes_sections<'a>(repo: &'a Repository) -> Res<impl Iterator<Item = Item> + 'a> {
    let all_remotes = create_reference_items(repo, Reference::is_remote)?;
    let mut remotes = BTreeMap::new();
    for (name, remote) in all_remotes {
        let name =
            String::from_utf8_lossy(&repo.branch_remote_name(&name).map_err(Error::GetRemote)?)
                .to_string();

        match remotes.entry(name) {
            Entry::Vacant(entry) => {
                entry.insert(vec![remote]);
            }
            Entry::Occupied(mut entry) => {
                entry.get_mut().push(remote);
            }
        }
    }

    Ok(remotes.into_iter().flat_map(move |(name, items)| {
        vec![
            items::blank_line(),
            Item {
                id: hash(&name),
                section: true,
                depth: 0,
                target_data: Some(TargetData::Header(SectionHeader::Remote(name))),
                ..Default::default()
            },
        ]
        .into_iter()
        .chain(items)
    }))
}

fn create_tags_section<'a>(repo: &'a Repository) -> Res<impl Iterator<Item = Item> + 'a> {
    let mut tags = create_reference_items(repo, Reference::is_tag)?;
    Ok(match tags.next() {
        Some((_name, item)) => vec![
            items::blank_line(),
            Item {
                id: hash("tags"),
                section: true,
                depth: 0,
                target_data: Some(TargetData::Header(SectionHeader::Tags)),
                ..Default::default()
            },
            item,
        ],
        None => vec![],
    }
    .into_iter()
    .chain(tags.map(|(_name, item)| item)))
}

fn create_reference_items<'a, F>(
    repo: &'a Repository,
    filter: F,
) -> Res<impl Iterator<Item = (String, Item)> + 'a>
where
    F: FnMut(&Reference<'a>) -> bool + 'a,
{
    Ok(repo
        .references()
        .map_err(Error::ListGitReferences)?
        .filter_map(Result::ok)
        .filter(filter)
        .map(move |reference| {
            let name = reference.name().unwrap().to_owned();
            let shorthand = reference.shorthand().unwrap().to_owned();

            let prefix = create_prefix(repo, &reference);

            // FIXME this is most likely wrong since this shorthand is used
            //       in other contexts where the prefix would mess things up
            let shorthand = format!("{prefix}{shorthand}");

            let ref_kind = if reference.is_branch() {
                RefKind::Branch(shorthand)
            } else if reference.is_tag() {
                RefKind::Tag(shorthand)
            } else {
                unreachable!()
            };

            let item = Item {
                id: hash(&name),
                depth: 1,
                target_data: Some(TargetData::Reference(ref_kind)),
                ..Default::default()
            };
            (name, item)
        }))
}

fn create_prefix(repo: &Repository, reference: &Reference) -> &'static str {
    let head = repo.head().ok();

    if repo.head_detached().unwrap_or(false) {
        if reference.target() == head.as_ref().and_then(Reference::target) {
            "? "
        } else {
            "  "
        }
    } else if reference.name() == head.as_ref().and_then(Reference::name) {
        "* "
    } else {
        "  "
    }
}
