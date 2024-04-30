use super::{create_prompt_with_default, selected_rev, Action, OpTrait};
use crate::{items::TargetData, menu::arg::Arg, screen, state::State, term::Term, Res};
use derive_more::Display;
use git2::Oid;
use std::{ffi::OsString, rc::Rc};

pub(crate) const ARGS: &[Arg] = &[
    Arg::new_int_opt("-n", "Limit number of commits", Some(256)),
    // Arg::new_str("-S", "Search occurences"), // TOOD: Implement search
];

#[derive(Display)]
#[display(fmt = "Log current")]
pub(crate) struct LogCurrent;
impl OpTrait for LogCurrent {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|state: &mut State, _term: &mut Term| {
            goto_log_screen(state, None);
            Ok(())
        }))
    }
}

#[derive(Display)]
#[display(fmt = "Log other")]
pub(crate) struct LogOther;
impl OpTrait for LogOther {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(create_prompt_with_default(
            "Log rev",
            log_other,
            selected_rev,
        ))
    }
}

fn log_other(state: &mut State, _term: &mut Term, _args: &[OsString], result: &str) -> Res<()> {
    let oid = match state.repo.revparse_single(result) {
        Ok(rev) => Ok(rev.id()),
        Err(err) => Err(format!("Failed due to: {:?}", err.code())),
    }?;

    goto_log_screen(state, Some(oid));
    Ok(())
}

fn goto_log_screen(state: &mut State, rev: Option<Oid>) {
    state.screens.drain(1..);
    let size = state.screens.last().unwrap().size;
    let limit = state
        .pending_menu
        .as_ref()
        .map(|m| m.args.get("-n"))
        .flatten()
        .map(Arg::get_i32)
        .flatten()
        .unwrap_or(i32::MAX);

    state.screens.push(
        screen::log::create(
            Rc::clone(&state.config),
            Rc::clone(&state.repo),
            size,
            limit as usize,
            rev,
        )
        .expect("Couldn't create screen"),
    );
}
