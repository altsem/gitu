use super::{create_prompt, Action, OpTrait};
use crate::{
    git::remote::{get_push_remote, get_upstream_shortname},
    items::TargetData,
    menu::arg::Arg,
    state::State,
    term::Term,
    Res,
};
use std::{process::Command, rc::Rc};

pub(crate) fn init_args() -> Vec<Arg> {
    vec![Arg::new_flag("--rebase", "Rebase local commits", false)]
}

pub(crate) struct PullFromPushRemote;
impl OpTrait for PullFromPushRemote {
    fn get_action(&self, target: Option<&TargetData>) -> Option<Action> {
        todo!("Implement PullFromPushRemote");
    }

    fn display(&self, state: &State) -> String {
        match get_push_remote(&state.repo) {
            Ok(Some(remote)) => format!("from {}", remote),
            Ok(None) => "pushRemote".into(),
            Err(e) => format!("error: {}", e),
        }
    }
}

pub(crate) struct PullFromUpstream;
impl OpTrait for PullFromUpstream {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|state: &mut State, term: &mut Term| {
            todo!("Implement PullFromUpstream");
            // let mut cmd = Command::new("git");
            // cmd.arg("pull");
            // cmd.args(state.pending_menu.as_ref().unwrap().args());

            // state.close_menu();
            // state.run_cmd_async(term, &[], cmd)?;
            // Ok(())
        }))
    }

    fn display(&self, state: &State) -> String {
        match get_upstream_shortname(&state.repo) {
            Ok(Some(upstream)) => format!("from {}", upstream),
            Ok(None) => "upstream".into(),
            Err(e) => format!("error: {}", e),
        }
    }
}

pub(crate) struct PullFromElsewhere;
impl OpTrait for PullFromElsewhere {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(create_prompt("Select remote", pull_elsewhere, true))
    }

    fn display(&self, _state: &State) -> String {
        "from elsewhere".into()
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
