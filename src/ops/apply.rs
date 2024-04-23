use std::process::Command;

use super::{cmd, Action, OpTrait};
use crate::{git::diff::PatchMode, items::TargetData};
use derive_more::*;

#[derive(Display)]
#[display(fmt = "Apply")]
pub(crate) struct Apply;
impl OpTrait for Apply {
    fn get_action(&self, target: Option<&TargetData>) -> Option<Action> {
        let action = match target.cloned() {
            Some(TargetData::Hunk(h)) => cmd(h.format_patch().into_bytes(), apply_cmd),
            Some(TargetData::HunkLine(h, i)) => cmd(
                h.format_line_patch(i..(i + 1), PatchMode::Normal)
                    .into_bytes(),
                apply_cmd,
            ),
            _ => return None,
        };

        Some(action)
    }
    fn is_target_op(&self) -> bool {
        true
    }
}

fn apply_cmd() -> Command {
    let mut cmd = Command::new("git");
    cmd.args(["apply", "--recount"]);
    cmd
}
