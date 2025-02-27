use super::OpTrait;
use crate::{git::diff::PatchMode, items::TargetData, state::State, term::Term, Action};
use std::{ffi::OsString, process::Command, rc::Rc};

pub(crate) struct Unstage;
impl OpTrait for Unstage {
    fn get_action(&self, target: Option<&TargetData>) -> Option<Action> {
        let action = match target.cloned() {
            Some(TargetData::AllStaged) => unstage_staged(),
            Some(TargetData::Delta { diff, file_i }) => {
                unstage_file(diff.text[diff.file_diffs[file_i].header.new_file.clone()].into())
            }
            Some(TargetData::Hunk {
                diff,
                file_i,
                hunk_i,
            }) => unstage_patch(diff.format_patch(file_i, hunk_i).into_bytes()),
            Some(TargetData::HunkLine {
                diff,
                file_i,
                hunk_i,
                line_i,
            }) => unstage_line(
                diff.format_line_patch(file_i, hunk_i, line_i..(line_i + 1), PatchMode::Reverse)
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
    Rc::new(move |state: &mut State, term: &mut Term| {
        let mut cmd = Command::new("git");
        cmd.args(["reset", "HEAD", "--"]);

        state.close_menu();
        state.run_cmd(term, &[], cmd)
    })
}

fn unstage_file(file: OsString) -> Action {
    Rc::new(move |state: &mut State, term: &mut Term| {
        let mut cmd = Command::new("git");
        cmd.args(["restore", "--staged"]);
        cmd.arg(&file);

        state.close_menu();
        state.run_cmd(term, &[], cmd)
    })
}

fn unstage_patch(input: Vec<u8>) -> Action {
    Rc::new(move |state: &mut State, term: &mut Term| {
        let mut cmd = Command::new("git");
        cmd.args(["apply", "--cached", "--reverse"]);

        state.close_menu();
        state.run_cmd(term, &input, cmd)
    })
}

fn unstage_line(input: Vec<u8>) -> Action {
    Rc::new(move |state: &mut State, term: &mut Term| {
        let mut cmd = Command::new("git");
        cmd.args(["apply", "--cached", "--reverse", "--recount"]);

        state.close_menu();
        state.run_cmd(term, &input, cmd)
    })
}
