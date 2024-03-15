use super::{cmd, cmd_arg, Action, OpTrait};
use crate::{git, items::TargetData, prompt::PromptData, state::State, term::Term, ErrorBuffer};
use derive_more::Display;
use std::{path::PathBuf, rc::Rc};
use tui_prompts::State as _;

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Discard")]
pub(crate) struct Discard;
impl OpTrait for Discard {
    fn get_action(&self, target: Option<&TargetData>) -> Option<Action> {
        let mut action = match target.cloned() {
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

        let update_fn = Rc::new(move |state: &mut State, term: &mut Term| {
            if state.prompt.state.status().is_pending() {
                match state.prompt.state.value() {
                    "y" => {
                        Rc::get_mut(&mut action).unwrap()(state, term)?;
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
        });

        Some(Rc::new(move |state: &mut State, _term: &mut Term| {
            state.prompt.set(PromptData {
                // TODO Show what is being discarded
                prompt_text: "Really discard? (y or n)".into(),
                update_fn: update_fn.clone(),
            });

            Ok(())
        }))
    }

    fn is_target_op(&self) -> bool {
        true
    }
}
