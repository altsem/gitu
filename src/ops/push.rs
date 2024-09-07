use super::{create_prompt, Action, OpTrait};
use crate::git::remote::{
    get_push_remote, get_upstream_components, set_push_remote,
};
use crate::{items::TargetData, menu::arg::Arg, state::State, term::Term, Res};
use derive_more::Display;
use std::{process::Command, rc::Rc};

pub(crate) fn init_args() -> Vec<Arg> {
    vec![
        Arg::new_flag("--force-with-lease", "Force with lease", false),
        Arg::new_flag("--force", "Force", false),
        Arg::new_flag("--no-verify", "Disable hooks", false),
        Arg::new_flag("--dry-run", "Dry run", false),
        Arg::new_flag("--set-upstream", "Set upstream", false),
    ]
}

#[derive(Display)]
#[display(fmt = "to pushRemote")]
pub(crate) struct PushRemote;
impl OpTrait for PushRemote {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(
            |state: &mut State, term: &mut Term| match get_push_remote(&state.repo)? {
                None => {
                    let mut prompt =
                        create_prompt("Set pushRemote then push", set_push_remote_and_push, true);
                    Rc::get_mut(&mut prompt).unwrap()(state, term)
                }
                Some(push_remote) => push_elsewhere(state, term, &push_remote),
            },
        ))
    }
}

fn set_push_remote_and_push(state: &mut State, term: &mut Term, push_remote_name: &str) -> Res<()> {
    let repo = state.repo.clone();
    let push_remote = repo
        .find_remote(push_remote_name)
        .map_err(|_| "Invalid pushRemote")?;
    set_push_remote(&repo, Some(&push_remote)).map_err(|_| "Could not set pushRemote config")?;
    push_elsewhere(state, term, push_remote_name)
}

#[derive(Display)]
#[display(fmt = "to upstream")]
pub(crate) struct Push;

impl OpTrait for Push {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|state: &mut State, term: &mut Term| {
            let repo = state.repo.clone();
            let upstream = get_upstream_components(&repo)?;
            match upstream {
                None => {
                    let mut prompt =
                        create_prompt("Set upstream then push", set_upstream_and_push, true);
                    Rc::get_mut(&mut prompt).unwrap()(state, term)
                }
                Some((remote, branch)) => {
                    push_elsewhere_with_branch(state, term, &remote, Some(&branch))
                }
            }
        }))
    }
}

fn set_upstream_and_push(state: &mut State, term: &mut Term, upstream_name: &str) -> Res<()> {
    state
        .pending_menu
        .as_mut()
        .unwrap()
        .args
        .get_mut("--set-upstream")
        .ok_or("Internal error")?
        .set("")?;
    let repo = state.repo.clone();
    let head = repo.head()?;
    let branch = if head.is_branch() {
        head.shorthand().ok_or("Branch is not valid UTF-8")?
    } else {
        return Err("Head is not a branch".into())
    };
    push_elsewhere_with_branch(state, term, upstream_name, Some(branch))
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
    push_elsewhere_with_branch(state, term, remote, None)
}

fn push_elsewhere_with_branch(
    state: &mut State,
    term: &mut Term,
    remote: &str,
    branch: Option<&str>,
) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.args(["push"]);
    cmd.args(state.pending_menu.as_ref().unwrap().args());
    cmd.arg(remote);
    if let Some(branch) = branch {
        cmd.arg(branch);
    }

    state.close_menu();
    state.run_cmd_async(term, &[], cmd)?;
    Ok(())
}
