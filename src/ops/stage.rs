use super::OpTrait;
use crate::{
    app::{App, State},
    git::diff::{Diff, PatchMode},
    item_data::ItemData,
    term::Term,
    Action,
};
use std::{ffi::OsString, process::Command, rc::Rc};

pub(crate) struct Stage;
impl OpTrait for Stage {
    fn get_action(&self, target: &ItemData) -> Option<Action> {
        let action = match target {
            ItemData::AllUnstaged => stage_unstaged(),
            // FIXME can we avoid clone?
            ItemData::AllUntracked(untracked) => stage_untracked(untracked.clone()),
            ItemData::File(u) => stage_file(u.into()),
            ItemData::Delta { diff, file_i } => {
                stage_file(diff.text[diff.file_diffs[*file_i].header.new_file.clone()].into())
            }
            ItemData::Hunk {
                diff,
                file_i,
                hunk_i,
            } => stage_patch(Rc::clone(diff), *file_i, *hunk_i),
            ItemData::HunkLine {
                diff,
                file_i,
                hunk_i,
                line_i,
            } => stage_line(Rc::clone(diff), *file_i, *hunk_i, *line_i),
            _ => return None,
        };

        Some(action)
    }

    fn is_target_op(&self) -> bool {
        true
    }

    fn display(&self, _state: &State) -> String {
        "Stage".into()
    }
}

fn stage_unstaged() -> Action {
    Rc::new(move |app: &mut App, term: &mut Term| {
        let mut cmd = Command::new("git");
        cmd.args(["add", "-u", "."]);

        app.close_menu();
        app.run_cmd(term, &[], cmd)
    })
}

fn stage_untracked(untracked: Vec<std::path::PathBuf>) -> Action {
    Rc::new(move |app: &mut App, term: &mut Term| {
        let mut cmd = Command::new("git");
        cmd.arg("add");
        cmd.args(untracked.clone());

        app.close_menu();
        app.run_cmd(term, &[], cmd)
    })
}

fn stage_file(file: OsString) -> Action {
    Rc::new(move |app, term| {
        let mut cmd = Command::new("git");
        cmd.args(["add"]);
        cmd.arg(&file);

        app.close_menu();
        app.run_cmd(term, &[], cmd)
    })
}

fn stage_patch(diff: Rc<Diff>, file_i: usize, hunk_i: usize) -> Action {
    Rc::new(move |app, term| {
        let mut cmd = Command::new("git");
        cmd.args(["apply", "--cached"]);

        app.close_menu();
        app.run_cmd(
            term,
            &diff.format_hunk_patch(file_i, hunk_i).into_bytes(),
            cmd,
        )
    })
}

fn stage_line(diff: Rc<Diff>, file_i: usize, hunk_i: usize, line_i: usize) -> Action {
    Rc::new(move |app, term| {
        let mut cmd = Command::new("git");
        cmd.args(["apply", "--cached", "--recount"]);

        let input = diff
            .format_line_patch(file_i, hunk_i, line_i..(line_i + 1), PatchMode::Normal)
            .into_bytes();

        app.close_menu();
        app.run_cmd(term, &input, cmd)
    })
}
