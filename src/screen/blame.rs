use std::{rc::Rc, sync::Arc};

use crate::{
    Res,
    config::Config,
    git, highlight,
    item_data::{BlameFile, ItemData, SectionHeader},
    items::{Item, hash},
};
use git2::Repository;
use ratatui::layout::Size;

use super::Screen;

pub(crate) fn create(
    config: Arc<Config>,
    repo: Rc<Repository>,
    size: Size,
    file_path: String,
    commit: Option<String>,
    target_line: Option<u32>,
) -> Res<Screen> {
    let mut screen = Screen::new(
        Arc::clone(&config),
        size,
        Box::new(move || {
            let commit_display = commit.as_deref().unwrap_or("HEAD").to_string();
            let blame_lines = git::blame(repo.as_ref(), &file_path, commit.as_deref())?;

            let full_content = blame_lines
                .iter()
                .map(|l| l.content.as_str())
                .collect::<Vec<_>>()
                .join("\n");

            let highlights = highlight::highlight_blame_file(&config, &file_path, full_content);

            let blame_file = Rc::new(BlameFile { highlights });

            let header = Item {
                id: hash(("blame_header", file_path.as_str(), commit_display.as_str())),
                depth: 0,
                unselectable: true,
                data: ItemData::Header(SectionHeader::Blame(file_path.clone(), commit_display)),
                ..Default::default()
            };

            let entries = blame_lines
                .into_iter()
                .enumerate()
                .scan(None::<String>, |prev_hash, (line_i, line)| {
                    let is_new_chunk = prev_hash.as_deref() != Some(line.commit_hash.as_str());
                    *prev_hash = Some(line.commit_hash.clone());
                    Some((line_i, line, is_new_chunk))
                })
                .flat_map(|(line_i, line, is_new_chunk)| {
                    let mut items = Vec::new();

                    if is_new_chunk {
                        items.push(Item {
                            id: hash(("blame_chunk", line.commit_hash.as_str(), line.line_num)),
                            depth: 0,
                            data: ItemData::BlameHeader {
                                commit_hash: line.commit_hash.clone(),
                                short_hash: line.short_hash.clone(),
                                _author: line.author.clone(),
                                _author_time: line.author_time,
                                summary: line.summary.clone(),
                                file_path: file_path.clone(),
                                line_num: line.orig_line_num,
                                blamed_line_num: line.line_num,
                            },
                            ..Default::default()
                        });
                    }

                    items.push(Item {
                        id: hash(("blame_code", line.commit_hash.as_str(), line.line_num)),
                        depth: 0,
                        data: ItemData::BlameCodeLine {
                            blame_file: Rc::clone(&blame_file),
                            line_i,
                            line_num: line.line_num,
                            orig_line_num: line.orig_line_num,
                            content: line.content,
                            commit_hash: line.commit_hash,
                            file_path: file_path.clone(),
                        },
                        ..Default::default()
                    });

                    items
                });

            Ok(std::iter::once(header).chain(entries).collect())
        }),
    )?;

    if let Some(line) = target_line
        && !screen.select_matching(
            |data| matches!(data, ItemData::BlameCodeLine { line_num, .. } if *line_num == line),
        )
    {
        screen.select_last_matching(|data| {
            matches!(data, ItemData::BlameHeader { blamed_line_num, .. }
                if *blamed_line_num <= line)
        });
    }

    Ok(screen)
}
