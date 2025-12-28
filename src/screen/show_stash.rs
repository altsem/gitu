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
            let details = commit.details.lines();

            let git::StashDiffs {
                staged,
                unstaged,
                untracked,
            } = git::stash_diffs(repo.as_ref(), &stash_ref)?;

            let mut out: Vec<Item> = Vec::new();
            out.extend(iter::once(Item {
                id: hash(["stash_section", &commit.hash]),
                depth: 0,
                data: ItemData::Header(SectionHeader::Commit(commit.hash.clone())),
                ..Default::default()
            }));
            out.extend(details.into_iter().map(|line| Item {
                id: hash(["stash", &commit.hash]),
                depth: 1,
                unselectable: true,
                data: ItemData::Raw(line.to_string()),
                ..Default::default()
            }));

            let push_diff_section = |out: &mut Vec<Item>, header: SectionHeader, diff| {
                let diff = Rc::new(diff);
                out.extend([
                    items::blank_line(),
                    Item {
                        id: hash(["stash_diff_section", &commit.hash, &format!("{header:?}")]),
                        depth: 0,
                        data: ItemData::Header(header),
                        ..Default::default()
                    },
                ]);
                out.extend(items::create_diff_items(&diff, 1, false));
            };

            if !staged.file_diffs.is_empty() {
                push_diff_section(
                    &mut out,
                    SectionHeader::StagedChanges(staged.file_diffs.len()),
                    staged,
                );
            }

            if !unstaged.file_diffs.is_empty() {
                push_diff_section(
                    &mut out,
                    SectionHeader::UnstagedChanges(unstaged.file_diffs.len()),
                    unstaged,
                );
            }

            if let Some(untracked) = untracked
                && !untracked.file_diffs.is_empty()
            {
                push_diff_section(
                    &mut out,
                    SectionHeader::UntrackedFiles(untracked.file_diffs.len()),
                    untracked,
                );
            }

            Ok(out)
        }),
    )
}
