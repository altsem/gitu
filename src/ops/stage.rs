use super::{cmd, cmd_arg, OpTrait};
use crate::{git::diff::PatchMode, items::TargetData, state::State, term::Term, Action};
use derive_more::Display;
use std::{ffi::OsStr, process::Command, rc::Rc};

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Stage")]
pub(crate) struct Stage;
impl OpTrait for Stage {
    fn get_action(&self, target: Option<&TargetData>) -> Option<Action> {
        let action = match target.cloned() {
            Some(TargetData::AllUnstaged) => stage_unstaged(),
            Some(TargetData::AllUntracked(untracked)) => stage_untracked(untracked),
            Some(TargetData::File(u)) => cmd_arg(stage_file_cmd, u.into()),
            Some(TargetData::Delta(d)) => cmd_arg(stage_file_cmd, d.new_file.into()),
            Some(TargetData::Hunk(h)) => cmd(h.format_patch().into_bytes(), stage_patch_cmd),
            Some(TargetData::HunkLine(h, i)) => cmd(
                h.format_line_patch(i..(i + 1), PatchMode::Normal)
                    .into_bytes(),
                stage_line_cmd,
            ),
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

fn stage_file_cmd(file: &OsStr) -> Command {
    let mut cmd = Command::new("git");
    cmd.args(["add"]);
    cmd.arg(file);
    cmd
}

fn stage_patch_cmd() -> Command {
    let mut cmd = Command::new("git");
    cmd.args(["apply", "--cached"]);
    cmd
}

fn stage_line_cmd() -> Command {
    let mut cmd = Command::new("git");
    cmd.args(["apply", "--cached", "--recount"]);
    cmd
}
