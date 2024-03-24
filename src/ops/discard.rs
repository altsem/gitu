use super::{cmd, cmd_arg, Action, OpTrait};
use crate::{git, items::TargetData, state::State, term::Term};
use derive_more::Display;
use std::{path::PathBuf, rc::Rc};

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Discard")]
pub(crate) struct Discard;
impl OpTrait for Discard {
    fn get_action(&self, target: Option<&TargetData>) -> Option<Action> {
        let action = match target.cloned() {
            Some(TargetData::Branch(r)) => cmd_arg(git::discard_branch, r.into()),
            Some(TargetData::File(f)) => Rc::new(move |state: &mut State, _term: &mut Term| {
                let path = PathBuf::from_iter([
                    state.repo.workdir().expect("No workdir").to_path_buf(),
                    f.clone(),
                ]);
                std::fs::remove_file(path)?;
                state.screen_mut().update()
            }),
            Some(TargetData::Delta(d)) => {
                if d.old_file == d.new_file {
                    cmd_arg(git::checkout_file_cmd, d.old_file.into())
                } else {
                    // TODO Discard file move
                    return None;
                }
            }
            Some(TargetData::Hunk(h)) => cmd(
                h.format_patch().into_bytes(),
                git::discard_unstaged_patch_cmd,
            ),
            _ => return None,
        };

        Some(super::create_y_n_prompt(action, "Really discard?"))
    }

    fn is_target_op(&self) -> bool {
        true
    }
}
