use super::OpTrait;
use crate::{
    Action,
    app::{App, State},
    git::diff::{Diff, PatchMode},
    item_data::ItemData,
    term::Term,
};
use std::{process::Command, rc::Rc};

pub(crate) struct Reverse;
impl OpTrait for Reverse {
    fn get_action(&self, target: &ItemData) -> Option<Action> {
        let action = match target {
            ItemData::Delta { diff, file_i } => reverse_patch(diff.format_file_patch(*file_i)),
            ItemData::Hunk {
                diff,
                file_i,
                hunk_i,
            } => reverse_patch(diff.format_hunk_patch(*file_i, *hunk_i)),
            ItemData::HunkLine {
                diff,
                file_i,
                hunk_i,
                line_i,
                ..
            } => reverse_line(diff, *file_i, *hunk_i, *line_i),
            _ => return None,
        };

        Some(action)
    }

    fn is_target_op(&self) -> bool {
        true
    }

    fn display(&self, _state: &State) -> String {
        "Reverse".into()
    }
}

fn reverse_patch(patch: String) -> Action {
    let patch = patch.into_bytes();

    Rc::new(move |app: &mut App, term: &mut Term| {
        let mut cmd = Command::new("git");
        cmd.args(["apply", "--reverse"]);
        app.run_cmd(term, &patch, cmd)
    })
}

fn reverse_line(diff: &Rc<Diff>, file_i: usize, hunk_i: usize, line_i: usize) -> Action {
    let patch = diff
        .format_line_patch(file_i, hunk_i, line_i..(line_i + 1), PatchMode::Reverse)
        .into_bytes();

    Rc::new(move |app: &mut App, term: &mut Term| {
        let mut cmd = Command::new("git");
        cmd.args(["apply", "--reverse", "--recount"]);
        app.run_cmd(term, &patch, cmd)
    })
}
