use super::{cmd, cmd_arg, TargetOpTrait};
use crate::{git, items::TargetData, Action};
use ratatui::backend::Backend;

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Unstage;
impl<B: Backend> TargetOpTrait<B> for Unstage {
    fn get_action(&self, target: TargetData) -> Option<Action<B>> {
        match target {
            TargetData::Delta(d) => cmd_arg(git::unstage_file_cmd, d.new_file.into()),
            TargetData::Hunk(h) => cmd(h.format_patch().into_bytes(), git::unstage_patch_cmd),
            _ => None,
        }
    }
}
