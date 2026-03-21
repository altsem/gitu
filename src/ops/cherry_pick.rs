use std::{process::Command, rc::Rc};

use crate::{
    Res,
    app::{App, PromptParams, State},
    item_data::{ItemData, Rev},
    menu::arg::Arg,
    term::Term,
};

use super::{Action, OpTrait, selected_rev};

pub(crate) fn init_args() -> Vec<Arg> {
    vec![
        Arg::new_flag("--no-commit", "Don't commit", false),
        Arg::new_flag("--signoff", "Add Signed-off-by lines", false),
        Arg::new_flag("--edit", "Edit commit message", false),
    ]
}

pub(crate) struct CherryPickAbort;
impl OpTrait for CherryPickAbort {
    fn get_action(&self, _target: &ItemData) -> Option<Action> {
        Some(Rc::new(|app: &mut App, term: &mut Term| {
            let mut cmd = Command::new("git");
            cmd.args(["cherry-pick", "--abort"]);
            app.run_cmd_interactive(term, cmd)?;
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "Abort".into()
    }
}

pub(crate) struct CherryPickContinue;
impl OpTrait for CherryPickContinue {
    fn get_action(&self, _target: &ItemData) -> Option<Action> {
        Some(Rc::new(|app: &mut App, term: &mut Term| {
            let mut cmd = Command::new("git");
            cmd.args(["cherry-pick", "--continue"]);
            app.run_cmd_interactive(term, cmd)?;
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "Continue".into()
    }
}

pub(crate) struct CherryPick;
impl OpTrait for CherryPick {
    fn get_action(&self, _target: &ItemData) -> Option<Action> {
        Some(Rc::new(move |app: &mut App, term: &mut Term| {
            let commit = app.prompt(
                term,
                &PromptParams {
                    prompt: "Cherry-pick commit",
                    create_default_value: Box::new(|app| {
                        selected_rev(app)
                            .as_ref()
                            .map(Rev::shorthand)
                            .map(String::from)
                    }),
                    ..Default::default()
                },
            )?;

            cherry_pick(app, term, &commit)?;
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "Cherry-pick commit(s)".into()
    }
}

fn cherry_pick(app: &mut App, term: &mut Term, input: &str) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.arg("cherry-pick");
    cmd.args(app.state.pending_menu.as_ref().unwrap().args());
    cmd.arg(input);
    app.run_cmd_interactive(term, cmd)
}
