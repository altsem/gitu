use super::OpTrait;
use crate::{
    app::{App, State},
    git::diff::PatchMode,
    item_data::ItemData,
    term::Term,
    Action,
};
use std::{ffi::OsString, process::Command, rc::Rc};

pub(crate) struct Unstage;
impl OpTrait for Unstage {
    fn get_action(&self, target: &ItemData) -> Option<Action> {
        let action = match target {
            ItemData::AllStaged(_) => unstage_staged(),
            ItemData::Delta { diff, file_i } => {
                unstage_file(diff.text[diff.file_diffs[*file_i].header.new_file.clone()].into())
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

fn unstage_file(file: OsString) -> Action {
    Rc::new(move |app: &mut App, term: &mut Term| {
        let mut cmd = Command::new("git");
        cmd.args(["restore", "--staged"]);
        cmd.arg(&file);

        app.close_menu();
        app.run_cmd(term, &[], cmd)
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
