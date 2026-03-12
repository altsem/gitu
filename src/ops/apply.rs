use super::OpTrait;
use crate::{
    Action,
    app::{App, State},
    git::diff::{Diff, PatchMode},
    item_data::ItemData,
    term::Term,
};
use std::{process::Command, rc::Rc};

pub(crate) struct Apply;
impl OpTrait for Apply {
    fn get_action(&self, target: &ItemData) -> Option<Action> {
        let action = match target {
            ItemData::Stash { stash_ref, .. } => apply_stash(stash_ref.clone()),
            ItemData::Delta { diff, file_i } => apply_patch(diff.format_file_patch(*file_i)),
            ItemData::Hunk {
                diff,
                file_i,
                hunk_i,
            } => apply_patch(diff.format_hunk_patch(*file_i, *hunk_i)),
            ItemData::HunkLine {
                diff,
                file_i,
                hunk_i,
                line_i,
                ..
            } => apply_line(diff, *file_i, *hunk_i, *line_i),
            _ => return None,
        };

        Some(action)
    }

    fn is_target_op(&self) -> bool {
        true
    }

    fn display(&self, _state: &State) -> String {
        "Apply".into()
    }
}

fn apply_stash(stash_ref: String) -> Action {
    Rc::new(move |app: &mut App, term: &mut Term| {
        let mut cmd = Command::new("git");
        cmd.args(["stash", "apply", "-q"]);
        cmd.arg(&stash_ref);

        app.close_menu();
        app.run_cmd(term, &[], cmd)
    })
}

fn apply_line(diff: &Rc<Diff>, file_i: usize, hunk_i: usize, line_i: usize) -> Action {
    let patch = diff
        .format_line_patch(file_i, hunk_i, line_i..(line_i + 1), PatchMode::Normal)
        .into_bytes();

    Rc::new(move |app: &mut App, term: &mut Term| {
        let mut cmd = Command::new("git");
        cmd.args(["apply", "--recount"]);

        app.close_menu();
        app.run_cmd(term, &patch, cmd)
    })
}

fn apply_patch(patch: String) -> Action {
    let patch = patch.into_bytes();

    Rc::new(move |app: &mut App, term: &mut Term| {
        let mut cmd = Command::new("git");
        cmd.arg("apply");

        app.close_menu();
        app.run_cmd(term, &patch, cmd)
    })
}
