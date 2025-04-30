use super::{Action, OpTrait};
use crate::{
    items::TargetData,
    menu::arg::Arg,
    state::{PromptParams, State},
    term::Term,
    Res,
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
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|state, term| {
            let mut cmd = Command::new("git");
            cmd.args(["fetch", "--all", "--jobs", "10"]);
            cmd.args(state.pending_menu.as_ref().unwrap().args());

            state.close_menu();
            state.run_cmd_async(term, &[], cmd)?;
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "from all remotes".into()
    }
}

pub(crate) struct FetchElsewhere;
impl OpTrait for FetchElsewhere {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(move |state: &mut State, term: &mut Term| {
            let remote = state.prompt(
                term,
                &PromptParams {
                    prompt: "Select remote",
                    ..Default::default()
                },
            )?;

            push_elsewhere(state, term, &remote)?;
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "from elsewhere".into()
    }
}

fn push_elsewhere(state: &mut State, term: &mut Term, remote: &str) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.args(["fetch"]);
    cmd.args(state.pending_menu.as_ref().unwrap().args());
    cmd.arg(remote);

    state.close_menu();
    state.run_cmd_async(term, &[], cmd)?;
    Ok(())
}
