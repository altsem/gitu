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
        // -m Replay merge relative to parent (--mainline=)
        Arg::new_flag("--edit", "Edit commit message", true),
        Arg::new_flag("--no-edit", "Don't edit commit message", false),
        // =s Strategy (--strategy=)
        Arg::new_flag("--signoff", "Add Signed-off-by lines", false),
    ]
}

pub(crate) struct RevertAbort;
impl OpTrait for RevertAbort {
    fn get_action(&self, _target: &ItemData) -> Option<Action> {
        Some(Rc::new(|app: &mut App, term: &mut Term| {
            let mut cmd = Command::new("git");
            cmd.args(["revert", "--abort"]);
            app.run_cmd_interactive(term, cmd)?;
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "Abort".into()
    }
}

pub(crate) struct RevertContinue;
impl OpTrait for RevertContinue {
    fn get_action(&self, _target: &ItemData) -> Option<Action> {
        Some(Rc::new(|app: &mut App, term: &mut Term| {
            let mut cmd = Command::new("git");
            cmd.args(["revert", "--continue"]);
            app.run_cmd_interactive(term, cmd)?;
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "Continue".into()
    }
}

pub(crate) struct RevertCommit;
impl OpTrait for RevertCommit {
    fn get_action(&self, _target: &ItemData) -> Option<Action> {
        Some(Rc::new(move |app: &mut App, term: &mut Term| {
            let commit = app.prompt(
                term,
                &PromptParams {
                    prompt: "Revert commit",
                    create_default_value: Box::new(|app| {
                        selected_rev(app)
                            .as_ref()
                            .map(Rev::shorthand)
                            .map(String::from)
                    }),
                    ..Default::default()
                },
            )?;

            revert_commit(app, term, &commit)?;
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "Revert commit(s)".into()
    }
}

fn revert_commit(app: &mut App, term: &mut Term, input: &str) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.args(["revert"]);
    cmd.args(app.state.pending_menu.as_ref().unwrap().args());
    cmd.arg(input);
    app.run_cmd_interactive(term, cmd)
}
