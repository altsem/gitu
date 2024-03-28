use super::{Action, OpTrait};
use crate::{items::TargetData, state::State, term::Term};
use derive_more::Display;
use std::{process::Command, rc::Rc};

pub(crate) fn args() -> &'static [(&'static str, bool)] {
    &[]
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Pull")]
pub(crate) struct Pull;
impl OpTrait for Pull {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|state: &mut State, term: &mut Term| {
            let mut cmd = Command::new("git");
            cmd.args(["pull"]);

            state.run_cmd(term, &[], cmd)?;
            Ok(())
        }))
    }
}
