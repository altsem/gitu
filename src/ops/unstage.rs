use super::{cmd, cmd_arg, OpTrait};
use crate::{git, items::TargetData, Action};
use derive_more::Display;

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Unstage")]
pub(crate) struct Unstage;
impl OpTrait for Unstage {
    fn get_action(&self, target: Option<&TargetData>) -> Option<Action> {
        let action = match target.cloned() {
            Some(TargetData::Delta(d)) => cmd_arg(git::unstage_file_cmd, d.new_file.into()),
            Some(TargetData::Hunk(h)) => cmd(h.format_patch().into_bytes(), git::unstage_patch_cmd),
            _ => return None,
        };

        Some(action)
    }
    fn is_target_op(&self) -> bool {
        true
    }
}
