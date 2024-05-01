use super::{create_prompt, Action, OpTrait};
use crate::{items::TargetData, menu::arg::Arg, state::State, term::Term, Res};
use derive_more::Display;
use std::{ffi::OsString, process::Command, rc::Rc};

pub(crate) const ARGS: &[Arg] = &[
    Arg::new("--prune", "Prune deleted branches", false),
    Arg::new("--tags", "Fetch all tags", false),
];

#[derive(Display)]
#[display(fmt = "Fetch all")]
pub(crate) struct FetchAll;
impl OpTrait for FetchAll {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|state, term| {
            let mut cmd = Command::new("git");
            cmd.args(["fetch", "--all", "--jobs", "10"]);
            cmd.args(state.pending_menu.as_ref().unwrap().args());

            state.run_cmd_async(term, &[], cmd)?;
            Ok(())
        }))
    }
}

#[derive(Display)]
#[display(fmt = "Fetch from elsewhere")]
pub(crate) struct FetchElsewhere;
impl OpTrait for FetchElsewhere {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(create_prompt("Select remote", push_elsewhere))
    }
}

fn push_elsewhere(state: &mut State, term: &mut Term, args: &[OsString], remote: &str) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.args(["fetch"]);
    cmd.arg(remote);
    cmd.args(args);

    state.run_cmd_async(term, &[], cmd)?;
    Ok(())
}
