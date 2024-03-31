use super::{Action, OpTrait};
use crate::items::TargetData;
use derive_more::Display;
use std::{process::Command, rc::Rc};

pub(crate) fn args() -> &'static [(&'static str, bool)] {
    &[]
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Fetch all")]
pub(crate) struct FetchAll;
impl OpTrait for FetchAll {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|state, term| {
            let mut cmd = Command::new("git");
            cmd.args(["fetch", "--all", "--jobs", "10"]);

            state.run_cmd_async(term, &[], cmd)?;
            Ok(())
        }))
    }
}
