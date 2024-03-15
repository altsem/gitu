use super::{Action, OpTrait};
use crate::items::TargetData;
use derive_more::Display;
use std::{process::Command, rc::Rc};

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Fetch all")]
pub(crate) struct FetchAll;
impl OpTrait for FetchAll {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|state, term| {
            let mut cmd = Command::new("git");
            cmd.args(["fetch", "--all"]);

            state.run_external_cmd(term, &[], cmd)?;
            Ok(())
        }))
    }
}
