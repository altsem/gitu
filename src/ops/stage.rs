use super::{cmd, cmd_arg, OpTrait};
use crate::{git, items::TargetData, state::State, term::Term, Action};
use derive_more::Display;
use std::{process::Command, rc::Rc};

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Stage")]
pub(crate) struct Stage;
impl OpTrait for Stage {
    fn get_action(&self, target: Option<&TargetData>) -> Option<Action> {
        let action = match target.cloned() {
            Some(TargetData::AllUnstaged) => stage_unstaged(),
            Some(TargetData::AllUntracked(untracked)) => stage_untracked(untracked),
            Some(TargetData::File(u)) => cmd_arg(git::stage_file_cmd, u.into()),
            Some(TargetData::Delta(d)) => cmd_arg(git::stage_file_cmd, d.new_file.into()),
            Some(TargetData::Hunk(h)) => cmd(h.format_patch().into_bytes(), git::stage_patch_cmd),
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
        state.run_external_cmd(term, &[], cmd)
    })
}

fn stage_untracked(untracked: Vec<std::path::PathBuf>) -> Action {
    Rc::new(move |state: &mut State, term: &mut Term| {
        let mut cmd = Command::new("git");
        cmd.arg("add");
        cmd.args(untracked.clone());
        state.run_external_cmd(term, &[], cmd)
    })
}
