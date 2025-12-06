use super::{Action, OpTrait};
use crate::{
    Res,
    app::{App, PromptParams, State},
    error::Error,
    git::remote::{get_push_remote, get_upstream_components, get_upstream_remote, set_push_remote},
    item_data::ItemData,
    menu::arg::Arg,
    term::Term,
};
use std::{process::Command, rc::Rc};

pub(crate) fn init_args() -> Vec<Arg> {
    vec![
        Arg::new_flag("--prune", "Prune deleted branches", false),
        Arg::new_flag("--tags", "Fetch all tags", false),
    ]
}

pub(crate) struct FetchAll;
impl OpTrait for FetchAll {
    fn get_action(&self, _target: &ItemData) -> Option<Action> {
        Some(Rc::new(|app, term| {
            let mut cmd = Command::new("git");
            cmd.args(["fetch", "--all", "--jobs", "10"]);
            cmd.args(app.state.pending_menu.as_ref().unwrap().args());

            app.close_menu();
            app.run_cmd_async(term, &[], cmd)?;
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "from all remotes".into()
    }
}

pub(crate) struct FetchElsewhere;
impl OpTrait for FetchElsewhere {
    fn get_action(&self, _target: &ItemData) -> Option<Action> {
        Some(Rc::new(move |app: &mut App, term: &mut Term| {
            let remote = app.prompt(
                term,
                &PromptParams {
                    prompt: "Select remote",
                    ..Default::default()
                },
            )?;

            fetch_elsewhere(app, term, &remote)?;
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "from elsewhere".into()
    }
}

fn fetch_elsewhere(app: &mut App, term: &mut Term, remote: &str) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.args(["fetch"]);
    cmd.args(app.state.pending_menu.as_ref().unwrap().args());
    cmd.arg(remote);

    app.close_menu();
    app.run_cmd_async(term, &[], cmd)?;
    Ok(())
}

fn set_upstream_and_fetch(app: &mut App, term: &mut Term, upstream_name: &str) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.args(["branch", "--set-upstream-to", upstream_name]);
    app.run_cmd(term, &[], cmd)?;

    let Some((remote, _branch)) = get_upstream_components(&app.state.repo)? else {
        return Ok(());
    };

    fetch_elsewhere(app, term, &remote)
}

fn set_push_remote_and_fetch(app: &mut App, term: &mut Term, push_remote_name: &str) -> Res<()> {
    let repo = app.state.repo.clone();
    let push_remote = repo
        .find_remote(push_remote_name)
        .map_err(Error::GetRemote)?;

    // TODO Would be nice to have the command visible in the log. Resort to `git config`?
    set_push_remote(&repo, Some(&push_remote))?;

    fetch_elsewhere(app, term, push_remote_name)
}

pub(crate) struct FetchPushRemote;
impl OpTrait for FetchPushRemote {
    fn get_action(&self, _target: &ItemData) -> Option<Action> {
        Some(Rc::new(
            |app: &mut App, term: &mut Term| match get_push_remote(&app.state.repo)? {
                None => {
                    let push_remote_name = app.prompt(
                        term,
                        &PromptParams {
                            prompt: "Set pushRemote then fetch",
                            ..Default::default()
                        },
                    )?;

                    set_push_remote_and_fetch(app, term, &push_remote_name)?;
                    Ok(())
                }
                Some(push_remote) => fetch_elsewhere(app, term, &push_remote),
            },
        ))
    }

    fn display(&self, state: &State) -> String {
        match get_push_remote(&state.repo) {
            Ok(Some(remote)) => format!("from {remote}"),
            Ok(None) => "from pushRemote, setting that".into(),
            Err(e) => format!("error: {e}"),
        }
    }
}

pub(crate) struct FetchUpstream;
impl OpTrait for FetchUpstream {
    fn get_action(&self, _target: &ItemData) -> Option<Action> {
        Some(Rc::new(
            |app: &mut App, term: &mut Term| match get_upstream_components(&app.state.repo)? {
                None => {
                    let mut prompt = Rc::new(move |app: &mut App, term: &mut Term| {
                        let upstream_name = app.prompt(
                            term,
                            &PromptParams {
                                prompt: "Set upstream then fetch",
                                ..Default::default()
                            },
                        )?;

                        set_upstream_and_fetch(app, term, &upstream_name)?;
                        Ok(())
                    });
                    Rc::get_mut(&mut prompt).unwrap()(app, term)
                }
                Some((remote, _branch)) => fetch_elsewhere(app, term, &remote),
            },
        ))
    }

    fn display(&self, state: &State) -> String {
        match get_upstream_remote(&state.repo) {
            Ok(Some(upstream)) => format!("from {upstream}"),
            Ok(None) => "from upstream, setting that".into(),
            Err(e) => format!("error: {e}"),
        }
    }
}
