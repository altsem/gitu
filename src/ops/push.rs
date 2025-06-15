use super::{Action, OpTrait};
use crate::app::App;
use crate::app::PromptParams;
use crate::app::State;
use crate::error::Error;
use crate::git;
use crate::git::remote::{
    get_push_remote, get_upstream_components, get_upstream_shortname, set_push_remote,
};
use crate::{items::TargetData, menu::arg::Arg, term::Term, Res};
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
            |app: &mut App, term: &mut Term| match get_push_remote(&app.state.repo)? {
                None => {
                    let push_remote_name = app.prompt(
                        term,
                        &PromptParams {
                            prompt: "Set pushRemote then push",
                            ..Default::default()
                        },
                    )?;

                    set_push_remote_and_push(app, term, &push_remote_name)?;
                    Ok(())
                }
                Some(push_remote) => {
                    let head_ref = git::get_head_name(&app.state.repo)?;
                    let refspec = format!("{0}:{0}", head_ref);
                    push(app, term, &[&push_remote, &refspec])
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

fn set_push_remote_and_push(app: &mut App, term: &mut Term, push_remote_name: &str) -> Res<()> {
    let repo = app.state.repo.clone();
    let push_remote = repo
        .find_remote(push_remote_name)
        .map_err(Error::GetRemote)?;

    // TODO Would be nice to have the command visible in the log. Resort to `git config`?
    set_push_remote(&repo, Some(&push_remote))?;

    let head_ref = git::get_head_name(&app.state.repo)?;
    let refspec = format!("{0}:{0}", head_ref);
    push(app, term, &[push_remote_name, &refspec])
}

pub(crate) struct PushToUpstream;
impl OpTrait for PushToUpstream {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(
            |app: &mut App, term: &mut Term| match get_upstream_components(&app.state.repo)? {
                None => {
                    let mut prompt = Rc::new(move |app: &mut App, term: &mut Term| {
                        let upstream_name = app.prompt(
                            term,
                            &PromptParams {
                                prompt: "Set upstream then push",
                                ..Default::default()
                            },
                        )?;

                        set_upstream_and_push(app, term, &upstream_name)?;
                        Ok(())
                    });
                    Rc::get_mut(&mut prompt).unwrap()(app, term)
                }
                Some((remote, branch)) => push_head_to(app, term, &remote, &branch),
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

fn set_upstream_and_push(app: &mut App, term: &mut Term, upstream_name: &str) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.args(["branch", "--set-upstream-to", upstream_name]);
    app.run_cmd(term, &[], cmd)?;

    let Some((remote, branch)) = get_upstream_components(&app.state.repo)? else {
        return Ok(());
    };

    push_head_to(app, term, &remote, &branch)
}

pub(crate) struct PushToElsewhere;
impl OpTrait for PushToElsewhere {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(move |app: &mut App, term: &mut Term| {
            let remote = app.prompt(
                term,
                &PromptParams {
                    prompt: "Select remote",
                    ..Default::default()
                },
            )?;

            push_elsewhere(app, term, &remote)?;
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "to elsewhere".into()
    }
}

fn push_elsewhere(app: &mut App, term: &mut Term, remote: &str) -> Res<()> {
    push(app, term, &[remote])
}

fn push_head_to(app: &mut App, term: &mut Term, remote: &str, branch: &str) -> Res<()> {
    let head_ref = git::get_head_name(&app.state.repo)?;
    let refspec = format!("{}:refs/heads/{}", head_ref, branch);
    push(app, term, &[remote, &refspec])
}

fn push(app: &mut App, term: &mut Term, extra_args: &[&str]) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.args(["push"]);
    cmd.args(app.state.pending_menu.as_ref().unwrap().args());
    cmd.args(extra_args);

    app.close_menu();
    app.run_cmd_async(term, &[], cmd)?;
    Ok(())
}
