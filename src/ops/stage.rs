use super::{cmd, cmd_arg, TargetOpTrait};
use crate::{git, items::TargetData, Action};
use ratatui::backend::Backend;

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Stage;
impl<B: Backend> TargetOpTrait<B> for Stage {
    fn get_action(&self, target: TargetData) -> Option<Action<B>> {
        match target {
            TargetData::File(u) => cmd_arg(git::stage_file_cmd, u.into()),
            TargetData::Delta(d) => cmd_arg(git::stage_file_cmd, d.new_file.into()),
            TargetData::Hunk(h) => cmd(h.format_patch().into_bytes(), git::stage_patch_cmd),
            _ => None,
        }
    }
}
