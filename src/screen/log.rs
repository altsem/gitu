use super::Screen;
use crate::{
    Res,
    config::Config,
    git::tree,
    item_data::ItemData,
    items::{self, Item, log},
};
use git2::{Oid, Repository};
use regex::Regex;
use std::{rc::Rc, sync::Arc};

pub(crate) fn create(
    config: Arc<Config>,
    repo: Rc<Repository>,
    size: (u16, u16),
    limit: usize,
    revs: Vec<Oid>,
    msg_regex: Option<Regex>,
) -> Res<Screen> {
    Screen::new(
        Arc::clone(&config),
        size,
        Box::new(move || match revs.as_slice() {
            [] => log(&repo, limit, None, msg_regex.clone()),
            [rev] => log(&repo, limit, Some(*rev), msg_regex.clone()),
            _ => {
                let rows = tree::tree_roots(&repo, limit, &revs)?;

                if rows.is_empty() {
                    Ok(vec![])
                } else {
                    Ok(rows.into_iter().enumerate().map(row_to_item).collect())
                }
            }
        }),
    )
}

/// Turns a tree row into a screen item. Commit rows are selectable; the
/// graph-only rows (merge branches, collapses) are unselectable padding.
fn row_to_item((i, row): (usize, tree::TreeRow)) -> Item {
    match row.commit {
        Some(commit) => Item {
            id: items::hash(commit.oid.clone()),
            depth: 1,
            data: ItemData::Commit {
                graph: row.graph,
                oid: commit.oid,
                short_id: commit.short_id,
                associated_references: commit.associated_references,
                summary: commit.summary,
                author: commit.author,
                age: commit.age,
            },
            ..Default::default()
        },
        None => Item {
            id: items::hash(format!("graph-{i}-{}", row.graph)),
            depth: 1,
            unselectable: true,
            data: ItemData::Raw(row.graph),
            ..Default::default()
        },
    }
}
