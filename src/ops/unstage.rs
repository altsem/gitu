use super::OpTrait;
use crate::{
    git::diff::{format_line_patch, format_patch, PatchMode},
    items::TargetData,
    state::State,
    term::Term,
    Action,
};
use core::str;
use std::{ffi::OsString, process::Command, rc::Rc};

pub(crate) struct Unstage;
impl OpTrait for Unstage {
    fn get_action(&self, target: Option<&TargetData>) -> Option<Action> {
        let action = match target.cloned() {
            Some(TargetData::AllStaged) => unstage_staged(),
            Some(TargetData::Delta { diff, file_i }) => unstage_file(
                str::from_utf8(&diff.text[diff.file_diffs[file_i].header.new_file.clone()])
                    .unwrap()
                    .into(),
            ),
            Some(TargetData::Hunk {
                diff,
                file_i,
                hunk_i,
            }) => unstage_patch(format_patch(&diff, file_i, hunk_i).into_bytes()),
            Some(TargetData::HunkLine {
                diff,
                file_i,
                hunk_i,
                line_i,
            }) => unstage_line(
                format_line_patch(
                    &diff,
                    file_i,
                    hunk_i,
                    line_i..(line_i + 1),
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
