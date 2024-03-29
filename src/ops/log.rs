use super::{create_rev_prompt, Action, OpTrait};
use crate::{items::TargetData, screen, state::State, term::Term, Res};
use derive_more::Display;
use git2::Oid;
use std::rc::Rc;

pub(crate) fn args() -> &'static [(&'static str, bool)] {
    &[]
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
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

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Log other")]
pub(crate) struct LogOther;
impl OpTrait for LogOther {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(create_rev_prompt("Log rev", log_other))
    }
}

fn log_other(state: &mut State, _term: &mut Term, result: &str) -> Res<()> {
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
    state.screens.push(
        screen::log::create(Rc::clone(&state.config), Rc::clone(&state.repo), size, rev)
            .expect("Couldn't create screen"),
    );
}
