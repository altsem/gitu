use super::{selected_rev, OpTrait};
use crate::{
    app::{App, PromptParams},
    menu::arg::Arg,
    item_data::ItemData,
    term::Term,
    Action, Res,
};
use std::{process::Command, rc::Rc};

pub(crate) fn init_args() -> Vec<Arg> {
    vec![]
}

pub(crate) struct ResetSoft;
impl OpTrait for ResetSoft {
    fn get_action(&self, _target: Option<&ItemData>) -> Option<Action> {
        Some(Rc::new(move |app: &mut App, term: &mut Term| {
            let rev = app.prompt(
                term,
                &PromptParams {
                    prompt: "Soft reset to",
                    create_default_value: Box::new(selected_rev),
                    ..Default::default()
                },
            )?;

            reset_soft(app, term, &rev)?;
            Ok(())
        }))
    }

    fn display(&self, _app: &App) -> String {
        "soft".into()
    }
}

fn reset_soft(app: &mut App, term: &mut Term, input: &str) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.args(["reset", "--soft"]);
    cmd.args(app.state.pending_menu.as_ref().unwrap().args());
    cmd.arg(input);

    app.close_menu();
    app.run_cmd(term, &[], cmd)
}

pub(crate) struct ResetMixed;
impl OpTrait for ResetMixed {
    fn get_action(&self, _target: Option<&ItemData>) -> Option<Action> {
        Some(Rc::new(move |app: &mut App, term: &mut Term| {
            let rev = app.prompt(
                term,
                &PromptParams {
                    prompt: "Mixed reset to",
                    create_default_value: Box::new(selected_rev),
                    ..Default::default()
                },
            )?;

            reset_mixed(app, term, &rev)?;
            Ok(())
        }))
    }

    fn display(&self, _app: &App) -> String {
        "mixed".into()
    }
}

fn reset_mixed(app: &mut App, term: &mut Term, input: &str) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.args(["reset", "--mixed"]);
    cmd.args(app.state.pending_menu.as_ref().unwrap().args());
    cmd.arg(input);

    app.close_menu();
    app.run_cmd(term, &[], cmd)
}

pub(crate) struct ResetHard;
impl OpTrait for ResetHard {
    fn get_action(&self, _target: Option<&ItemData>) -> Option<Action> {
        Some(Rc::new(move |app: &mut App, term: &mut Term| {
            let rev = app.prompt(
                term,
                &PromptParams {
                    prompt: "Hard reset to",
                    create_default_value: Box::new(selected_rev),
                    ..Default::default()
                },
            )?;

            reset_hard(app, term, &rev)?;
            Ok(())
        }))
    }

    fn display(&self, _app: &App) -> String {
        "hard".into()
    }
}

fn reset_hard(app: &mut App, term: &mut Term, input: &str) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.args(["reset", "--hard"]);
    cmd.args(app.state.pending_menu.as_ref().unwrap().args());
    cmd.arg(input);

    app.close_menu();
    app.run_cmd(term, &[], cmd)
}
