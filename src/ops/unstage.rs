use super::OpTrait;
use crate::{git::diff::PatchMode, items::TargetData, state::State, term::Term, Action};
use derive_more::Display;
use std::{ffi::OsString, process::Command, rc::Rc};

#[derive(Display)]
#[display(fmt = "Unstage")]
pub(crate) struct Unstage;
impl OpTrait for Unstage {
    fn get_action(&self, target: Option<&TargetData>) -> Option<Action> {
        let action = match target.cloned() {
            Some(TargetData::AllStaged) => unstage_staged(),
            Some(TargetData::Delta(d)) => unstage_file(d.new_file.into()),
            Some(TargetData::Hunk(h)) => unstage_patch(h.format_patch().into_bytes()),
            Some(TargetData::HunkLine(h, i)) => unstage_line(
                h.format_line_patch(i..(i + 1), PatchMode::Reverse)
                    .into_bytes(),
            ),
            _ => return None,
        };

        Some(action)
    }
    fn is_target_op(&self) -> bool {
        true
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
