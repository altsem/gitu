use super::{create_prompt, Action, OpTrait};
use crate::{items::TargetData, menu::arg::Arg, state::State, term::Term, Res};
use derive_more::Display;
use std::{process::Command, rc::Rc};

pub(crate) fn init_args() -> Vec<Arg> {
    vec![Arg::new_flag("--rebase", "Rebase local commits", false)]
}

#[derive(Display)]
#[display(fmt = "from default")]
pub(crate) struct Pull;
impl OpTrait for Pull {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|state: &mut State, term: &mut Term| {
            let mut cmd = Command::new("git");
            cmd.arg("pull");
            cmd.args(state.pending_menu.as_ref().unwrap().args());

            state.close_menu();
            state.run_cmd_async(term, &[], cmd)?;
            Ok(())
        }))
    }
}

#[derive(Display)]
#[display(fmt = "from elsewhere")]
pub(crate) struct PullElsewhere;
impl OpTrait for PullElsewhere {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(create_prompt("Select remote", pull_elsewhere, true))
    }
}

fn pull_elsewhere(state: &mut State, term: &mut Term, remote: &str) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.args(["pull"]);
    cmd.args(state.pending_menu.as_ref().unwrap().args());
    cmd.arg(remote);

    state.close_menu();
    state.run_cmd_async(term, &[], cmd)?;
    Ok(())
}
