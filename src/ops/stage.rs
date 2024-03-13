use super::{cmd, cmd_arg, TargetOpTrait};
use crate::{git, items::TargetData, Action};

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Stage;
impl TargetOpTrait for Stage {
    fn get_action(&self, target: TargetData) -> Option<Action> {
        match target {
            TargetData::File(u) => cmd_arg(git::stage_file_cmd, u.into()),
            TargetData::Delta(d) => cmd_arg(git::stage_file_cmd, d.new_file.into()),
            TargetData::Hunk(h) => cmd(h.format_patch().into_bytes(), git::stage_patch_cmd),
            _ => None,
        }
    }
}
