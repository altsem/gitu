use super::OpTrait;
use crate::{
    git::diff::{Hunk, PatchMode},
    items::TargetData,
    state::State,
    term::Term,
    Action,
};
use derive_more::Display;
use std::{ffi::OsString, process::Command, rc::Rc};

#[derive(Display)]
#[display(fmt = "Stage")]
pub(crate) struct Stage;
impl OpTrait for Stage {
    fn get_action(&self, target: Option<&TargetData>) -> Option<Action> {
        let action = match target.cloned() {
            Some(TargetData::AllUnstaged) => stage_unstaged(),
            Some(TargetData::AllUntracked(untracked)) => stage_untracked(untracked),
            Some(TargetData::File(u)) => stage_file(u.into()),
            Some(TargetData::Delta(d)) => stage_file(d.new_file.into()),
            Some(TargetData::Hunk(h)) => stage_patch(h),
            Some(TargetData::HunkLine(h, i)) => stage_line(h, i),
            _ => return None,
        };

        Some(action)
    }
    fn is_target_op(&self) -> bool {
        true
    }
}

fn stage_unstaged() -> Action {
    Rc::new(move |state: &mut State, term: &mut Term| {
        let mut cmd = Command::new("git");
        cmd.args(["add", "-u", "."]);

        state.run_cmd(term, &[], cmd)
    })
}

fn stage_untracked(untracked: Vec<std::path::PathBuf>) -> Action {
    Rc::new(move |state: &mut State, term: &mut Term| {
        let mut cmd = Command::new("git");
        cmd.arg("add");
        cmd.args(untracked.clone());

        state.run_cmd(term, &[], cmd)
    })
}

fn stage_file(file: OsString) -> Action {
    Rc::new(move |state, term| {
        let mut cmd = Command::new("git");
        cmd.args(["add"]);
        cmd.arg(&file);

        state.run_cmd(term, &[], cmd)
    })
}

fn stage_patch(h: Rc<Hunk>) -> Action {
    Rc::new(move |state, term| {
        let mut cmd = Command::new("git");
        cmd.args(["apply", "--cached"]);

        state.run_cmd(term, &h.format_patch().into_bytes(), cmd)
    })
}

fn stage_line(h: Rc<Hunk>, i: usize) -> Action {
    Rc::new(move |state, term| {
        let mut cmd = Command::new("git");
        cmd.args(["apply", "--cached", "--recount"]);

        let input = h
            .format_line_patch(i..(i + 1), PatchMode::Normal)
            .into_bytes();
        state.run_cmd(term, &input, cmd)
    })
}
