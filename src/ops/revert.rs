use std::{process::Command, rc::Rc};

use crate::{
    app::{App, PromptParams, State},
    menu::arg::Arg,
    target_data::TargetData,
    term::Term,
    Res,
};

use super::{selected_rev, Action, OpTrait};

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
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|app: &mut App, term: &mut Term| {
            let mut cmd = Command::new("git");
            cmd.args(["revert", "--abort"]);

            app.close_menu();
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
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|app: &mut App, term: &mut Term| {
            let mut cmd = Command::new("git");
            cmd.args(["revert", "--continue"]);

            app.close_menu();
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
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(move |app: &mut App, term: &mut Term| {
            let commit = app.prompt(
                term,
                &PromptParams {
                    prompt: "Revert commit",
                    create_default_value: Box::new(selected_rev),
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

    app.close_menu();
    app.run_cmd_interactive(term, cmd)
}
