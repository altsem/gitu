use super::{cmd, cmd_arg, TargetOpTrait};
use crate::{git, items::TargetData, Action};
use derive_more::Display;

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Stage")]
pub(crate) struct Stage;
impl TargetOpTrait for Stage {
    fn get_action(&self, target: Option<&TargetData>) -> Option<Action> {
        match target.cloned() {
            Some(TargetData::File(u)) => cmd_arg(git::stage_file_cmd, u.into()),
            Some(TargetData::Delta(d)) => cmd_arg(git::stage_file_cmd, d.new_file.into()),
            Some(TargetData::Hunk(h)) => cmd(h.format_patch().into_bytes(), git::stage_patch_cmd),
            _ => None,
        }
    }
}
