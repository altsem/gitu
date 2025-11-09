use super::OpTrait;
use crate::{
    Action,
    app::{App, State},
    git::{self, diff::PatchMode},
    gitu_diff::Status,
    item_data::ItemData,
    term::Term,
};
use std::{path::PathBuf, process::Command, rc::Rc};

pub(crate) struct Unstage;
impl OpTrait for Unstage {
    fn get_action(&self, target: &ItemData) -> Option<Action> {
        let action = match target {
            ItemData::AllStaged(_) => unstage_staged(),
            ItemData::Delta { diff, file_i } => {
                let diff_header = &diff.file_diffs[*file_i].header;
                let file_path = match diff_header.status {
                    Status::Deleted => &diff_header.old_file,
                    _ => &diff_header.new_file,
                };

                unstage_file(file_path.fmt(&diff.text).into_owned().into())
            }
            ItemData::Hunk {
                diff,
                file_i,
                hunk_i,
            } => unstage_patch(diff.format_hunk_patch(*file_i, *hunk_i).into_bytes()),
            ItemData::HunkLine {
                diff,
                file_i,
                hunk_i,
                line_i,
                ..
            } => unstage_line(
                diff.format_line_patch(
                    *file_i,
                    *hunk_i,
                    *line_i..(*line_i + 1),
                    PatchMode::Reverse,
                )
                .into_bytes(),
            ),
            _ => return None,
        };

        Some(action)
    }
    fn is_target_op(&self) -> bool {
        true
    }

    fn display(&self, _state: &State) -> String {
        "Unstage".into()
    }
}

fn unstage_staged() -> Action {
    Rc::new(move |app: &mut App, term: &mut Term| {
        let mut cmd = Command::new("git");
        cmd.args(["reset", "HEAD", "--"]);

        app.close_menu();
        app.run_cmd(term, &[], cmd)
    })
}

fn unstage_file(file: PathBuf) -> Action {
    Rc::new(move |app: &mut App, term: &mut Term| {
        app.close_menu();
        app.run_cmd(term, &[], git::restore_index(&file))
    })
}

fn unstage_patch(input: Vec<u8>) -> Action {
    Rc::new(move |app: &mut App, term: &mut Term| {
        let mut cmd = Command::new("git");
        cmd.args(["apply", "--cached", "--reverse"]);

        app.close_menu();
        app.run_cmd(term, &input, cmd)
    })
}

fn unstage_line(input: Vec<u8>) -> Action {
    Rc::new(move |app: &mut App, term: &mut Term| {
        let mut cmd = Command::new("git");
        cmd.args(["apply", "--cached", "--reverse", "--recount"]);

        app.close_menu();
        app.run_cmd(term, &input, cmd)
    })
}
