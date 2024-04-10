use super::{Action, OpTrait};
use crate::{items::TargetData, menu::arg::Arg, state::State, term::Term};
use derive_more::Display;
use std::{process::Command, rc::Rc};

pub(crate) const ARGS: &[Arg] = &[
    Arg::new("--force-with-lease", "Force with lease", false),
    Arg::new("--force", "Force", false),
    Arg::new("--no-verify", "Disable hooks", false),
    Arg::new("--dry-run", "Dry run", false),
];

#[derive(Display)]
#[display(fmt = "Push")]
pub(crate) struct Push;
impl OpTrait for Push {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|state: &mut State, term: &mut Term| {
            let mut cmd = Command::new("git");
            cmd.args(["push"]);
            cmd.args(state.pending_menu.as_ref().unwrap().args());

            state.run_cmd_async(term, &[], cmd)?;
            Ok(())
        }))
    }
}
