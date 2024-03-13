use super::{cmd, cmd_arg, get_action, Action, Op, OpTrait, TargetOp, TargetOpTrait};
use crate::{git, items::TargetData, state::State, ErrorBuffer, Res};
use ratatui::{backend::Backend, Terminal};
use std::{borrow::Cow, path::PathBuf};
use tui_prompts::{prelude::Status, State as _};

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Discard;
impl<B: Backend> OpTrait<B> for Discard {
    fn trigger(&self, state: &mut State, _term: &mut Terminal<B>) -> Res<()> {
        state.prompt_action::<B>(Op::Target(TargetOp::Discard));
        Ok(())
    }

    fn format_prompt(&self, _state: &State) -> Cow<'static, str> {
        // TODO Show what is being discarded
        "Really discard? (y or n)".into()
    }

    fn prompt_update(&self, status: Status, state: &mut State, term: &mut Terminal<B>) -> Res<()> {
        if status.is_pending() {
            match state.prompt.state.value() {
                "y" => {
                    let mut action =
                        get_action(state.clone_target_data(), TargetOp::Discard).unwrap();
                    action(state, term)?;
                    state.prompt.reset(term)?;
                }
                "" => (),
                _ => {
                    state.error_buffer = Some(ErrorBuffer("Discard aborted".to_string()));
                    state.prompt.reset(term)?;
                }
            }
        }
        Ok(())
    }
}

impl<B: Backend> TargetOpTrait<B> for Discard {
    fn get_action(&self, target: TargetData) -> Option<Action<B>> {
        match target {
            TargetData::Branch(r) => cmd_arg(git::discard_branch, r.into()),
            TargetData::File(f) => Some(Box::new(move |state, _term| {
                let path = PathBuf::from_iter([
                    state.repo.workdir().expect("No workdir").to_path_buf(),
                    f.clone(),
                ]);
                std::fs::remove_file(path)?;
                state.screen_mut().update()
            })),
            TargetData::Delta(d) => {
                if d.old_file == d.new_file {
                    cmd_arg(git::checkout_file_cmd, d.old_file.into())
                } else {
                    // TODO Discard file move
                    None
                }
            }
            TargetData::Hunk(h) => cmd(
                h.format_patch().into_bytes(),
                git::discard_unstaged_patch_cmd,
            ),
            _ => None,
        }
    }
}
