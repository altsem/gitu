use super::{selected_rev, OpTrait};
use crate::{
    items::TargetData,
    menu::arg::Arg,
    state::{PromptParams, State},
    term::Term,
    Action, Res,
};
use std::{process::Command, rc::Rc};

pub(crate) fn init_args() -> Vec<Arg> {
    vec![]
}

pub(crate) struct ResetSoft;
impl OpTrait for ResetSoft {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(move |state: &mut State, term: &mut Term| {
            let rev = state.prompt(
                term,
                &PromptParams {
                    prompt: "Soft reset to",
                    create_default_value: Box::new(selected_rev),
                    ..Default::default()
                },
            )?;

            reset_soft(state, term, &rev)?;
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "soft".into()
    }
}

fn reset_soft(state: &mut State, term: &mut Term, input: &str) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.args(["reset", "--soft"]);
    cmd.args(state.pending_menu.as_ref().unwrap().args());
    cmd.arg(input);

    state.close_menu();
    state.run_cmd(term, &[], cmd)
}

pub(crate) struct ResetMixed;
impl OpTrait for ResetMixed {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(move |state: &mut State, term: &mut Term| {
            let rev = state.prompt(
                term,
                &PromptParams {
                    prompt: "Mixed reset to",
                    create_default_value: Box::new(selected_rev),
                    ..Default::default()
                },
            )?;

            reset_mixed(state, term, &rev)?;
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "mixed".into()
    }
}

fn reset_mixed(state: &mut State, term: &mut Term, input: &str) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.args(["reset", "--mixed"]);
    cmd.args(state.pending_menu.as_ref().unwrap().args());
    cmd.arg(input);

    state.close_menu();
    state.run_cmd(term, &[], cmd)
}

pub(crate) struct ResetHard;
impl OpTrait for ResetHard {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(move |state: &mut State, term: &mut Term| {
            let rev = state.prompt(
                term,
                &PromptParams {
                    prompt: "Hard reset to",
                    create_default_value: Box::new(selected_rev),
                    ..Default::default()
                },
            )?;

            reset_hard(state, term, &rev)?;
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "hard".into()
    }
}

fn reset_hard(state: &mut State, term: &mut Term, input: &str) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.args(["reset", "--hard"]);
    cmd.args(state.pending_menu.as_ref().unwrap().args());
    cmd.arg(input);

    state.close_menu();
    state.run_cmd(term, &[], cmd)
}
