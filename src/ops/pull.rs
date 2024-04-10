use super::{Action, OpTrait};
use crate::{items::TargetData, menu::arg::Arg, state::State, term::Term};
use derive_more::Display;
use std::{process::Command, rc::Rc};

pub(crate) const ARGS: &[Arg] = &[Arg::new("--rebase", "Rebase local commits", false)];

#[derive(Display)]
#[display(fmt = "Pull")]
pub(crate) struct Pull;
impl OpTrait for Pull {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|state: &mut State, term: &mut Term| {
            let mut cmd = Command::new("git");
            cmd.arg("pull");
            cmd.args(state.pending_menu.as_ref().unwrap().args());

            state.run_cmd_async(term, &[], cmd)?;
            Ok(())
        }))
    }
}
