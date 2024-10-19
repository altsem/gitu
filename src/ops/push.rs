use super::{create_prompt, Action, OpTrait};
use crate::git;
use crate::git::remote::{
    get_push_remote, get_upstream_components, get_upstream_shortname, set_push_remote,
};
use crate::{items::TargetData, menu::arg::Arg, state::State, term::Term, Res};
use std::{process::Command, rc::Rc};

pub(crate) fn init_args() -> Vec<Arg> {
    vec![
        Arg::new_flag("--force-with-lease", "Force with lease", false),
        Arg::new_flag("--force", "Force", false),
        Arg::new_flag("--no-verify", "Disable hooks", false),
        Arg::new_flag("--dry-run", "Dry run", false),
    ]
}

pub(crate) struct PushToPushRemote;
impl OpTrait for PushToPushRemote {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(
            |state: &mut State, term: &mut Term| match get_push_remote(&state.repo)? {
                None => {
                    let mut prompt =
                        create_prompt("Set pushRemote then push", set_push_remote_and_push, true);
                    Rc::get_mut(&mut prompt).unwrap()(state, term)
                }
                Some(push_remote) => {
                    let head_ref = git::get_head(&state.repo)?;
                    let refspec = format!("{0}:{0}", head_ref);
                    push(state, term, &[&push_remote, &refspec])
                }
            },
        ))
    }

    fn display(&self, state: &State) -> String {
        match get_push_remote(&state.repo) {
            Ok(Some(remote)) => format!("to {}", remote),
            Ok(None) => "pushRemote, setting that".into(),
            Err(e) => format!("error: {}", e),
        }
    }
}

fn set_push_remote_and_push(state: &mut State, term: &mut Term, push_remote_name: &str) -> Res<()> {
    let repo = state.repo.clone();
    let push_remote = repo
        .find_remote(push_remote_name)
        .map_err(|_| "Invalid pushRemote")?;

    // TODO Would be nice to have the command visible in the log. Resort to `git config`?
    set_push_remote(&repo, Some(&push_remote)).map_err(|_| "Could not set pushRemote config")?;

    let head_ref = git::get_head(&state.repo)?;
    let refspec = format!("{0}:{0}", head_ref);
    push(state, term, &[push_remote_name, &refspec])
}

pub(crate) struct PushToUpstream;
impl OpTrait for PushToUpstream {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(
            |state: &mut State, term: &mut Term| match get_upstream_components(&state.repo)? {
                None => {
                    let mut prompt =
                        create_prompt("Set upstream then push", set_upstream_and_push, true);
                    Rc::get_mut(&mut prompt).unwrap()(state, term)
                }
                Some((remote, branch)) => push_head_to(state, term, &remote, &branch),
            },
        ))
    }

    fn display(&self, state: &State) -> String {
        match get_upstream_shortname(&state.repo) {
            Ok(Some(upstream)) => format!("to {}", upstream),
            Ok(None) => "upstream, setting that".into(),
            Err(e) => format!("error: {}", e),
        }
    }
}

fn set_upstream_and_push(state: &mut State, term: &mut Term, upstream_name: &str) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.args(["branch", "--set-upstream-to", upstream_name]);
    state.run_cmd(term, &[], cmd)?;

    let Some((remote, branch)) = get_upstream_components(&state.repo)? else {
        return Ok(());
    };

    push_head_to(state, term, &remote, &branch)
}

pub(crate) struct PushToElsewhere;
impl OpTrait for PushToElsewhere {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(create_prompt("Select remote", push_elsewhere, true))
    }

    fn display(&self, _state: &State) -> String {
        "to elsewhere".into()
    }
}

fn push_elsewhere(state: &mut State, term: &mut Term, remote: &str) -> Res<()> {
    push(state, term, &[remote])
}

fn push_head_to(state: &mut State, term: &mut Term, remote: &str, branch: &str) -> Res<()> {
    let head_ref = git::get_head(&state.repo)?;
    let refspec = format!("{}:refs/heads/{}", head_ref, branch);
    push(state, term, &[remote, &refspec])
}

fn push(state: &mut State, term: &mut Term, extra_args: &[&str]) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.args(["push"]);
    cmd.args(state.pending_menu.as_ref().unwrap().args());
    cmd.args(extra_args);

    state.close_menu();
    state.run_cmd_async(term, &[], cmd)?;
    Ok(())
}
