use super::OpTrait;
use crate::{
    git::diff::{format_line_patch, format_patch, Diff, PatchMode},
    items::TargetData,
    state::State,
    term::Term,
    Action,
};
use core::str;
use std::{ffi::OsString, process::Command, rc::Rc};

pub(crate) struct Stage;
impl OpTrait for Stage {
    fn get_action(&self, target: Option<&TargetData>) -> Option<Action> {
        let action = match target.cloned() {
            Some(TargetData::AllUnstaged) => stage_unstaged(),
            Some(TargetData::AllUntracked(untracked)) => stage_untracked(untracked),
            Some(TargetData::File(u)) => stage_file(u.into()),
            Some(TargetData::Delta { diff, file_i }) => stage_file(
                str::from_utf8(&diff.text[diff.file_diffs[file_i].header.new_file.clone()])
                    .unwrap()
                    .into(),
            ),
            Some(TargetData::Hunk {
                diff,
                file_i,
                hunk_i,
            }) => stage_patch(diff, file_i, hunk_i),
            Some(TargetData::HunkLine {
                diff,
                file_i,
                hunk_i,
                line_i,
            }) => stage_line(diff, file_i, hunk_i, line_i),
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
    Rc::new(move |state: &mut State, term: &mut Term| {
        let mut cmd = Command::new("git");
        cmd.args(["add", "-u", "."]);

        state.close_menu();
        state.run_cmd(term, &[], cmd)
    })
}

fn stage_untracked(untracked: Vec<std::path::PathBuf>) -> Action {
    Rc::new(move |state: &mut State, term: &mut Term| {
        let mut cmd = Command::new("git");
        cmd.arg("add");
        cmd.args(untracked.clone());

        state.close_menu();
        state.run_cmd(term, &[], cmd)
    })
}

fn stage_file(file: OsString) -> Action {
    Rc::new(move |state, term| {
        let mut cmd = Command::new("git");
        cmd.args(["add"]);
        cmd.arg(&file);

        state.close_menu();
        state.run_cmd(term, &[], cmd)
    })
}

fn stage_patch(diff: Rc<Diff>, file_i: usize, hunk_i: usize) -> Action {
    Rc::new(move |state, term| {
        let mut cmd = Command::new("git");
        cmd.args(["apply", "--cached"]);

        state.close_menu();
        state.run_cmd(term, &format_patch(&diff, file_i, hunk_i).into_bytes(), cmd)
    })
}

fn stage_line(diff: Rc<Diff>, file_i: usize, hunk_i: usize, line_i: usize) -> Action {
    Rc::new(move |state, term| {
        let mut cmd = Command::new("git");
        cmd.args(["apply", "--cached", "--recount"]);

        let input = format_line_patch(
            &diff,
            file_i,
            hunk_i,
            line_i..(line_i + 1),
            PatchMode::Normal,
        )
        .into_bytes();

        state.close_menu();
        state.run_cmd(term, &input, cmd)
    })
}
