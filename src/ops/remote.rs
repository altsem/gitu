use std::{process::Command, rc::Rc};

use crate::{
    items::TargetData,
    state::{PromptParams, State},
    term::Term,
    Res,
};

use super::{Action, OpTrait};

pub(crate) struct AddRemote;

impl OpTrait for AddRemote {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(
            |state: &mut crate::state::State, term: &mut Term| {
                let remote_name = state.prompt(
                    term,
                    &PromptParams {
                        prompt: "Remote name",
                        ..Default::default()
                    },
                )?;

                let remote_url = state.prompt(
                    term,
                    &PromptParams {
                        prompt: "Remote url",
                        ..Default::default()
                    },
                )?;

                add_remote_with_name(state, term, &remote_name, &remote_url)?;

                Ok(())
            },
        ))
    }

    fn display(&self, _state: &State) -> String {
        "add remote".to_string()
    }
}

fn add_remote_with_name(
    state: &mut State,
    term: &mut Term,
    remote_name: &str,
    remote_url: &str,
) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.args(["remote", "add", remote_name, remote_url]);

    state.close_menu();
    state.run_cmd(term, &[], cmd)?;
    Ok(())
}
