use super::{selected_rev, Action, OpTrait};
use crate::{
    error::Error,
    items::TargetData,
    menu::arg::{any_regex, positive_number, Arg},
    screen,
    state::{PromptParams, State},
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
        // Arg::new_str("-S", "Search occurences"), // TOOD: Implement search
    ]
}

pub(crate) struct LogCurrent;
impl OpTrait for LogCurrent {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|state: &mut State, term: &mut Term| {
            let answer = state.prompt(term, "Are you sure you want to view the log?")?;
            log::debug!("Answer: {answer}");

            let answer2 = state.prompt(term, "Are you really sure you want to view the log?")?;
            log::debug!("Answer: {answer2}");

            todo!("Remove this");
            goto_log_screen(state, None);
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "current".into()
    }
}

pub(crate) struct LogOther;
impl OpTrait for LogOther {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(move |state: &mut State, _term: &mut Term| {
            state.set_prompt(PromptParams {
                prompt: "Log rev",
                on_success: Box::new(log_other),
                create_default_value: Box::new(selected_rev),
                hide_menu: true,
            });

            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "other".into()
    }
}

fn log_other(state: &mut State, _term: &mut Term, result: &str) -> Res<()> {
    let oid_result = match state.repo.revparse_single(result) {
        Ok(rev) => Ok(rev.id()),
        Err(err) => Err(Error::FindGitRev(err)),
    };

    if oid_result.is_err() {
        state.close_menu();
    }

    let oid = oid_result?;

    goto_log_screen(state, Some(oid));
    Ok(())
}

fn goto_log_screen(state: &mut State, rev: Option<Oid>) {
    state.screens.drain(1..);
    let size = state.screens.last().unwrap().size;
    let limit = *state
        .pending_menu
        .as_ref()
        .and_then(|m| m.args.get("-n"))
        .and_then(|arg| arg.value_as::<u32>())
        .unwrap_or(&u32::MAX);

    let msg_regex_menu = state
        .pending_menu
        .as_ref()
        .and_then(|m| m.args.get("--grep"));

    let msg_regex = msg_regex_menu.and_then(|arg| arg.value_as::<Regex>().cloned());

    state.close_menu();

    state.screens.push(
        screen::log::create(
            Rc::clone(&state.config),
            Rc::clone(&state.repo),
            size,
            limit as usize,
            rev,
            msg_regex,
        )
        .expect("Couldn't create screen"),
    );
}
