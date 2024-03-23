use super::{cmd, cmd_arg, OpTrait};
use crate::{
    git::{self, diff::PatchMode},
    items::TargetData,
    state::State,
    term::Term,
    Action,
};
use derive_more::Display;
use std::{process::Command, rc::Rc};

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Unstage")]
pub(crate) struct Unstage;
impl OpTrait for Unstage {
    fn get_action(&self, target: Option<&TargetData>) -> Option<Action> {
        let action = match target.cloned() {
            Some(TargetData::AllStaged) => unstage_staged(),
            Some(TargetData::Delta(d)) => cmd_arg(git::unstage_file_cmd, d.new_file.into()),
            Some(TargetData::Hunk(h)) => cmd(h.format_patch().into_bytes(), git::unstage_patch_cmd),
            Some(TargetData::HunkLine(h, i)) => cmd(
                h.format_line_patch(i..(i + 1), PatchMode::Reverse)
                    .into_bytes(),
                git::unstage_line_cmd,
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
        state.run_external_cmd(term, &[], cmd)
    })
}
