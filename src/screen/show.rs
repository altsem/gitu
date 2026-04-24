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
    reference: String,
    target: Option<(String, u32)>,
) -> Res<Screen> {
    let mut screen = Screen::new(
        Arc::clone(&config),
        size,
        Box::new(move || {
            let commit = git::show_summary(repo.as_ref(), &reference)?;
            let show = git::show(repo.as_ref(), &reference)?;
            let details = commit.details.lines();

            Ok(iter::once(Item {
                id: hash(["commit_section", &commit.hash]),
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
            .chain(items::create_diff_items(
                &Rc::new(show),
                0,
                false,
                Some(commit.hash.clone()),
            ))
            .collect())
        }),
    )?;

    if let Some((file, line_num)) = target {
        let found = screen.select_matching(|data| {
            if let ItemData::HunkLine {
                diff,
                file_i,
                hunk_i,
                line_i,
                line_range,
            } = data
            {
                if diff.file_diffs[*file_i].header.new_file.fmt(&diff.text) != file {
                    return false;
                }
                let line = &diff.hunk_content(*file_i, *hunk_i)[line_range.clone()];
                !line.starts_with('-')
                    && diff.hunk_line_new_file_num(*file_i, *hunk_i, *line_i) == line_num
            } else {
                false
            }
        });
        if !found {
            screen.select_matching(|data| {
                matches!(data, ItemData::Delta { diff, file_i, .. }
                    if diff.file_diffs[*file_i].header.new_file.fmt(&diff.text) == file)
            });
        }
    }

    Ok(screen)
}
