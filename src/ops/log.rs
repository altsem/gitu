use super::{selected_rev, Action, OpTrait};
use crate::{
    app::{App, PromptParams, State},
    error::Error,
    menu::arg::{any_regex, positive_number, Arg},
    screen,
    item_data::ItemData,
    term::Term,
    Res,
};
use git2::Oid;
use regex::Regex;
use std::rc::Rc;

pub(crate) fn init_args() -> Vec<Arg> {
    vec![
        Arg::new_arg(
            "-n",
            "Limit number of commits",
            Some(|| 256),
            positive_number,
        ),
        Arg::new_arg("--grep", "Search messages", None, any_regex),
        // Arg::new_str("-S", "Search occurrences"), // TODO: Implement search
    ]
}

pub(crate) struct LogCurrent;
impl OpTrait for LogCurrent {
    fn get_action(&self, _target: Option<&ItemData>) -> Option<Action> {
        Some(Rc::new(|app: &mut App, _term: &mut Term| {
            goto_log_screen(app, None);
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "current".into()
    }
}

pub(crate) struct LogOther;
impl OpTrait for LogOther {
    fn get_action(&self, _target: Option<&ItemData>) -> Option<Action> {
        Some(Rc::new(move |app: &mut App, term: &mut Term| {
            let rev = app.prompt(
                term,
                &PromptParams {
                    prompt: "Log rev",
                    create_default_value: Box::new(selected_rev),
                    ..Default::default()
                },
            )?;

            log_other(app, term, &rev)?;
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "other".into()
    }
}

fn log_other(app: &mut App, _term: &mut Term, result: &str) -> Res<()> {
    let oid_result = match app.state.repo.revparse_single(result) {
        Ok(rev) => Ok(rev.id()),
        Err(err) => Err(Error::FindGitRev(err)),
    };

    if oid_result.is_err() {
        app.close_menu();
    }

    let oid = oid_result?;

    goto_log_screen(app, Some(oid));
    Ok(())
}

fn goto_log_screen(app: &mut App, rev: Option<Oid>) {
    app.state.screens.drain(1..);
    let size = app.state.screens.last().unwrap().size;
    let limit = *app
        .state
        .pending_menu
        .as_ref()
        .and_then(|m| m.args.get("-n"))
        .and_then(|arg| arg.value_as::<u32>())
        .unwrap_or(&u32::MAX);

    let msg_regex_menu = app
        .state
        .pending_menu
        .as_ref()
        .and_then(|m| m.args.get("--grep"));

    let msg_regex = msg_regex_menu.and_then(|arg| arg.value_as::<Regex>().cloned());

    app.close_menu();

    app.state.screens.push(
        screen::log::create(
            Rc::clone(&app.state.config),
            Rc::clone(&app.state.repo),
            size,
            limit as usize,
            rev,
            msg_regex,
        )
        .expect("Couldn't create screen"),
    );
}
