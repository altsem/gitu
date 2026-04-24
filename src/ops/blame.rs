use super::OpTrait;
use crate::{Action, app::State, error::Error, item_data::ItemData, screen};
use std::{rc::Rc, sync::Arc};

pub(crate) struct Blame;
impl OpTrait for Blame {
    fn get_action(&self, target: &ItemData) -> Option<Action> {
        match target {
            ItemData::Delta {
                diff,
                file_i,
                commit,
            } => {
                let file_path = diff.file_diffs[*file_i]
                    .header
                    .new_file
                    .fmt(&diff.text)
                    .to_string();
                open_blame(file_path, commit.clone(), None)
            }
            ItemData::Hunk {
                diff,
                file_i,
                hunk_i,
            } => {
                let file_path = diff.file_diffs[*file_i]
                    .header
                    .new_file
                    .fmt(&diff.text)
                    .to_string();
                let line = diff.hunk_first_changed_line_num(*file_i, *hunk_i);
                open_blame(file_path, diff.commit.clone(), Some(line))
            }
            ItemData::HunkLine {
                diff,
                file_i,
                hunk_i,
                line_i,
                line_range,
            } => {
                let hunk_content = diff.hunk_content(*file_i, *hunk_i);
                let line_content = &hunk_content[line_range.clone()];
                if line_content.starts_with('-') {
                    return None;
                }
                let file_path = diff.file_diffs[*file_i]
                    .header
                    .new_file
                    .fmt(&diff.text)
                    .to_string();
                let line = diff.hunk_line_new_file_num(*file_i, *hunk_i, *line_i);
                open_blame(file_path, diff.commit.clone(), Some(line))
            }
            ItemData::BlameHeader {
                commit_hash,
                file_path,
                line_num,
                ..
            } if !commit_hash.chars().all(|c| c == '0') => {
                let parent = format!("{}^", commit_hash);
                open_blame(file_path.clone(), Some(parent), Some(*line_num))
            }
            ItemData::BlameCodeLine {
                commit_hash,
                file_path,
                orig_line_num,
                ..
            } if !commit_hash.chars().all(|c| c == '0') => {
                let parent = format!("{}^", commit_hash);
                open_blame(file_path.clone(), Some(parent), Some(*orig_line_num))
            }
            _ => None,
        }
    }

    fn is_target_op(&self) -> bool {
        true
    }

    fn display(&self, _state: &State) -> String {
        "Blame".into()
    }
}

fn open_blame(
    file_path: String,
    commit: Option<String>,
    target_line: Option<u32>,
) -> Option<Action> {
    Some(Rc::new(move |app, term| {
        app.state.screens.push(
            screen::blame::create(
                Arc::clone(&app.state.config),
                Rc::clone(&app.state.repo),
                term.size().map_err(Error::Term)?,
                file_path.clone(),
                commit.clone(),
                target_line,
            )
            .expect("Couldn't create blame screen"),
        );
        Ok(())
    }))
}
