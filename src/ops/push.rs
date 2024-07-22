use super::{create_prompt, Action, OpTrait};
use crate::{items::TargetData, menu::arg::Arg, state::State, term::Term, Res};
use derive_more::Display;
use std::{process::Command, rc::Rc};

pub(crate) fn init_args() -> Vec<Arg> {
    vec![
        Arg::new_flag("--force-with-lease", "Force with lease", false),
        Arg::new_flag("--force", "Force", false),
        Arg::new_flag("--no-verify", "Disable hooks", false),
        Arg::new_flag("--dry-run", "Dry run", false),
    ]
}

#[derive(Display)]
#[display(fmt = "to default")]
pub(crate) struct Push;
impl OpTrait for Push {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|state: &mut State, term: &mut Term| {
            let mut cmd = Command::new("git");
            cmd.args(["push"]);
            cmd.args(state.pending_menu.as_ref().unwrap().args());

            state.close_menu();
            state.run_cmd_async(term, &[], cmd)?;
            Ok(())
        }))
    }
}

#[derive(Display)]
#[display(fmt = "to elsewhere")]
pub(crate) struct PushElsewhere;
impl OpTrait for PushElsewhere {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(create_prompt("Select remote", push_elsewhere, true))
    }
}

fn push_elsewhere(state: &mut State, term: &mut Term, remote: &str) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.args(["push"]);
    cmd.arg(format!("--repo={}", remote));
    cmd.args(state.pending_menu.as_ref().unwrap().args());

    state.close_menu();
    state.run_cmd_async(term, &[], cmd)?;
    Ok(())
}
