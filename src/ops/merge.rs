use super::{Action, OpTrait, selected_rev};
use crate::{
    Res,
    app::{App, PromptParams, State},
    item_data::ItemData,
    menu::arg::Arg,
    term::Term,
};
use std::{process::Command, rc::Rc};

pub(crate) fn init_args() -> Vec<Arg> {
    vec![
        Arg::new_flag("--ff-only", "Fast-forward only", false),
        Arg::new_flag("--no-ff", "No fast-forward", false),
    ]
}

pub(crate) struct MergeContinue;
impl OpTrait for MergeContinue {
    fn get_action(&self, _target: &ItemData) -> Option<Action> {
        Some(Rc::new(|app: &mut App, term: &mut Term| {
            let mut cmd = Command::new("git");
            cmd.args(["merge", "--continue"]);

            app.close_menu();
            app.run_cmd_interactive(term, cmd)?;
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "continue".into()
    }
}

pub(crate) struct MergeAbort;
impl OpTrait for MergeAbort {
    fn get_action(&self, _target: &ItemData) -> Option<Action> {
        Some(Rc::new(|app: &mut App, term: &mut Term| {
            let mut cmd = Command::new("git");
            cmd.args(["merge", "--abort"]);

            app.close_menu();
            app.run_cmd_interactive(term, cmd)?;
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "abort".into()
    }
}

fn merge(app: &mut App, term: &mut Term, rev: &str) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.arg("merge");
    cmd.args(app.state.pending_menu.as_ref().unwrap().args());
    cmd.arg(rev);

    app.close_menu();
    app.run_cmd_interactive(term, cmd)?;
    Ok(())
}

pub(crate) struct Merge;
impl OpTrait for Merge {
    fn get_action(&self, _target: &ItemData) -> Option<Action> {
        Some(Rc::new(|app: &mut App, term: &mut Term| {
            let rev = app.prompt(
                term,
                &PromptParams {
                    prompt: "Merge",
                    create_default_value: Box::new(selected_rev),
                    ..Default::default()
                },
            )?;

            merge(app, term, &rev)?;
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "merge".into()
    }
}
