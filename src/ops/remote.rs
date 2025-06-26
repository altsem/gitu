use std::{process::Command, rc::Rc};

use crate::{
    app::{App, PromptParams, State},
    item_data::ItemData,
    term::Term,
    Res,
};

use super::{Action, OpTrait};

pub(crate) struct AddRemote;
impl OpTrait for AddRemote {
    fn get_action(&self, _target: &ItemData) -> Option<Action> {
        Some(Rc::new(|app: &mut App, term: &mut Term| {
            let remote_name = app.prompt(
                term,
                &PromptParams {
                    prompt: "Remote name",
                    ..Default::default()
                },
            )?;

            let remote_url = app.prompt(
                term,
                &PromptParams {
                    prompt: "Remote url",
                    ..Default::default()
                },
            )?;

            add_remote_with_name(app, term, &remote_name, &remote_url)?;

            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "add remote".to_string()
    }
}

pub(crate) struct RenameRemote;
impl OpTrait for RenameRemote {
    fn get_action(&self, _target: &ItemData) -> Option<Action> {
        Some(Rc::new(|app: &mut App, term: &mut Term| {
            let remote_name = app.prompt(
                term,
                &PromptParams {
                    prompt: "Rename remote",
                    ..Default::default()
                },
            )?;

            let new_remote_name = app.prompt(
                term,
                &PromptParams {
                    prompt: "Rename to",
                    ..Default::default()
                },
            )?;

            rename_remote(app, term, &remote_name, &new_remote_name)?;
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "rename remote".to_string()
    }
}

pub(crate) struct RemoveRemote;
impl OpTrait for RemoveRemote {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(
            |state: &mut crate::state::State, term: &mut Term| {
                let remote_name = state.prompt(
                    term,
                    &PromptParams {
                        prompt: "Delete remote",
                        ..Default::default()
                    },
                )?;

                state.confirm(term, "Really delete remote (y or n)")?;
                remove_remote(state, term, &remote_name)?;

                Ok(())
            },
        ))
    }

    fn display(&self, _state: &State) -> String {
        "remove remote".to_string()
    }
}

fn remove_remote(
    state: &mut State,
    term: &mut ratatui::Terminal<crate::term::TermBackend>,
    remote_name: &str,
) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.args(["remote", "remove", remote_name]);

    state.close_menu();
    state.run_cmd(term, &[], cmd)?;
    Ok(())
}

fn rename_remote(
    app: &mut App,
    term: &mut ratatui::Terminal<crate::term::TermBackend>,
    remote_name: &str,
    new_remote_name: &str,
) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.args(["remote", "rename", remote_name, new_remote_name]);

    app.close_menu();
    app.run_cmd(term, &[], cmd)?;
    Ok(())
}

fn add_remote_with_name(
    app: &mut App,
    term: &mut Term,
    remote_name: &str,
    remote_url: &str,
) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.args(["remote", "add", remote_name, remote_url]);

    app.close_menu();
    app.run_cmd(term, &[], cmd)?;
    Ok(())
}
