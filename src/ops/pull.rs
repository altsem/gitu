use super::{create_prompt, Action, OpTrait};
use crate::{
    error::Error,
    git::{
        self,
        remote::{self, get_push_remote, get_upstream_components, get_upstream_shortname},
    },
    items::TargetData,
    menu::arg::Arg,
    state::State,
    term::Term,
    Res,
};
use std::{process::Command, rc::Rc, str};

pub(crate) fn init_args() -> Vec<Arg> {
    vec![Arg::new_flag("--rebase", "Rebase local commits", false)]
}

pub(crate) struct PullFromPushRemote;
impl OpTrait for PullFromPushRemote {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(
            |state: &mut State, term: &mut Term| match get_push_remote(&state.repo)? {
                None => {
                    let mut prompt =
                        create_prompt("Set pushRemote then pull", set_push_remote_and_pull, true);
                    Rc::get_mut(&mut prompt).unwrap()(state, term)
                }
                Some(push_remote) => {
                    let refspec = git::get_head_name(&state.repo)?;
                    pull(state, term, &[&push_remote, &refspec])
                }
            },
        ))
    }

    fn display(&self, state: &State) -> String {
        match get_push_remote(&state.repo) {
            Ok(Some(remote)) => format!("from {}", remote),
            Ok(None) => "pushRemote, setting that".into(),
            Err(e) => format!("error: {}", e),
        }
    }
}

fn set_push_remote_and_pull(state: &mut State, term: &mut Term, push_remote_name: &str) -> Res<()> {
    let repo = state.repo.clone();
    let push_remote = repo
        .find_remote(push_remote_name)
        .map_err(Error::GetRemote)?;

    remote::set_push_remote(&repo, Some(&push_remote))?;

    let refspec = git::get_head_name(&repo)?;
    pull(state, term, &[push_remote_name, &refspec])
}

pub(crate) struct PullFromUpstream;
impl OpTrait for PullFromUpstream {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(
            |state: &mut State, term: &mut Term| match get_upstream_components(&state.repo)? {
                None => {
                    let mut prompt =
                        create_prompt("Set upstream then pull", set_upstream_and_pull, true);
                    Rc::get_mut(&mut prompt).unwrap()(state, term)
                }
                Some((remote, branch)) => {
                    let refspec = format!("refs/heads/{}", branch);
                    pull(state, term, &[&remote, &refspec])
                }
            },
        ))
    }

    fn display(&self, state: &State) -> String {
        match get_upstream_shortname(&state.repo) {
            Ok(Some(upstream)) => format!("from {}", upstream),
            Ok(None) => "upstream, setting that".into(),
            Err(e) => format!("error: {}", e),
        }
    }
}

fn set_upstream_and_pull(state: &mut State, term: &mut Term, upstream_name: &str) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.args(["branch", "--set-upstream-to", upstream_name]);
    state.run_cmd(term, &[], cmd)?;

    let Some((remote, branch)) = get_upstream_components(&state.repo)? else {
        return Ok(());
    };

    let refspec = format!("refs/heads/{}", branch);
    pull(state, term, &[&remote, &refspec])
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
    pull(state, term, &[remote])
}

fn pull(state: &mut State, term: &mut Term, extra_args: &[&str]) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.args(["pull"]);
    cmd.args(state.pending_menu.as_ref().unwrap().args());
    cmd.args(extra_args);

    state.close_menu();
    state.run_cmd_async(term, &[], cmd)?;
    Ok(())
}
