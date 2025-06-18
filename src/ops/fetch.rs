use super::{Action, OpTrait};
use crate::{
    app::{App, PromptParams, State},
    menu::arg::Arg,
    item_data::ItemData,
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
    fn get_action(&self, _target: Option<&ItemData>) -> Option<Action> {
        Some(Rc::new(|app, term| {
            let mut cmd = Command::new("git");
            cmd.args(["fetch", "--all", "--jobs", "10"]);
            cmd.args(app.state.pending_menu.as_ref().unwrap().args());

            app.close_menu();
            app.run_cmd_async(term, &[], cmd)?;
            Ok(())
        }))
    }

    fn display(&self, _app: &App) -> String {
        "from all remotes".into()
    }
}

pub(crate) struct FetchElsewhere;
impl OpTrait for FetchElsewhere {
    fn get_action(&self, _target: Option<&ItemData>) -> Option<Action> {
        Some(Rc::new(move |app: &mut App, term: &mut Term| {
            let remote = app.prompt(
                term,
                &PromptParams {
                    prompt: "Select remote",
                    ..Default::default()
                },
            )?;

            push_elsewhere(app, term, &remote)?;
            Ok(())
        }))
    }

    fn display(&self, _app: &App) -> String {
        "from elsewhere".into()
    }
}

fn push_elsewhere(app: &mut App, term: &mut Term, remote: &str) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.args(["fetch"]);
    cmd.args(app.state.pending_menu.as_ref().unwrap().args());
    cmd.arg(remote);

    app.close_menu();
    app.run_cmd_async(term, &[], cmd)?;
    Ok(())
}
