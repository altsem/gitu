use super::{Action, OpTrait};
use crate::{
    app::{App, PromptParams, State},
    error::Error,
    git::{
        self,
        remote::{self, get_push_remote, get_upstream_components, get_upstream_shortname},
    },
    menu::arg::Arg,
    item_data::ItemData,
    term::Term,
    Res,
};
use std::{process::Command, rc::Rc, str};

pub(crate) fn init_args() -> Vec<Arg> {
    vec![Arg::new_flag("--rebase", "Rebase local commits", false)]
}

pub(crate) struct PullFromPushRemote;
impl OpTrait for PullFromPushRemote {
    fn get_action(&self, _target: Option<&ItemData>) -> Option<Action> {
        Some(Rc::new(
            |app: &mut App, term: &mut Term| match get_push_remote(&app.state.repo)? {
                None => {
                    let push_remote_name = app.prompt(
                        term,
                        &PromptParams {
                            prompt: "Set pushRemote then pull",
                            ..Default::default()
                        },
                    )?;

                    set_push_remote_and_pull(app, term, &push_remote_name)?;
                    Ok(())
                }
                Some(push_remote) => {
                    let refspec = git::get_head_name(&app.state.repo)?;
                    pull(app, term, &[&push_remote, &refspec])
                }
            },
        ))
    }

    fn display(&self, app: &App) -> String {
        match get_push_remote(&app.state.repo) {
            Ok(Some(remote)) => format!("from {}", remote),
            Ok(None) => "pushRemote, setting that".into(),
            Err(e) => format!("error: {}", e),
        }
    }
}

fn set_push_remote_and_pull(app: &mut App, term: &mut Term, push_remote_name: &str) -> Res<()> {
    let repo = app.state.repo.clone();
    let push_remote = repo
        .find_remote(push_remote_name)
        .map_err(Error::GetRemote)?;

    remote::set_push_remote(&repo, Some(&push_remote))?;

    let refspec = git::get_head_name(&repo)?;
    pull(app, term, &[push_remote_name, &refspec])
}

pub(crate) struct PullFromUpstream;
impl OpTrait for PullFromUpstream {
    fn get_action(&self, _target: Option<&ItemData>) -> Option<Action> {
        Some(Rc::new(
            |app: &mut App, term: &mut Term| match get_upstream_components(&app.state.repo)? {
                None => {
                    let upstream_name = app.prompt(
                        term,
                        &PromptParams {
                            prompt: "Set upstream then pull",
                            ..Default::default()
                        },
                    )?;

                    set_upstream_and_pull(app, term, &upstream_name)?;
                    Ok(())
                }
                Some((remote, branch)) => {
                    let refspec = format!("refs/heads/{}", branch);
                    pull(app, term, &[&remote, &refspec])
                }
            },
        ))
    }

    fn display(&self, app: &App) -> String {
        match get_upstream_shortname(&app.state.repo) {
            Ok(Some(upstream)) => format!("from {}", upstream),
            Ok(None) => "upstream, setting that".into(),
            Err(e) => format!("error: {}", e),
        }
    }
}

fn set_upstream_and_pull(app: &mut App, term: &mut Term, upstream_name: &str) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.args(["branch", "--set-upstream-to", upstream_name]);
    app.run_cmd(term, &[], cmd)?;

    let Some((remote, branch)) = get_upstream_components(&app.state.repo)? else {
        return Ok(());
    };

    let refspec = format!("refs/heads/{}", branch);
    pull(app, term, &[&remote, &refspec])
}

pub(crate) struct PullFromElsewhere;
impl OpTrait for PullFromElsewhere {
    fn get_action(&self, _target: Option<&ItemData>) -> Option<Action> {
        Some(Rc::new(move |app: &mut App, term: &mut Term| {
            let remote = app.prompt(
                term,
                &PromptParams {
                    prompt: "Select remote",
                    ..Default::default()
                },
            )?;

            pull_elsewhere(app, term, &remote)?;
            Ok(())
        }))
    }

    fn display(&self, _app: &App) -> String {
        "from elsewhere".into()
    }
}

fn pull_elsewhere(app: &mut App, term: &mut Term, remote: &str) -> Res<()> {
    pull(app, term, &[remote])
}

fn pull(app: &mut App, term: &mut Term, extra_args: &[&str]) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.args(["pull"]);
    cmd.args(app.state.pending_menu.as_ref().unwrap().args());
    cmd.args(extra_args);

    app.close_menu();
    app.run_cmd_async(term, &[], cmd)?;
    Ok(())
}
