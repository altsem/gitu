use crate::{items::TargetData, state::State, term::Term, Res};
use std::{
    ffi::{OsStr, OsString},
    fmt::Display,
    process::Command,
    rc::Rc,
};

pub(crate) mod checkout;
pub(crate) mod commit;
pub(crate) mod discard;
pub(crate) mod editor;
pub(crate) mod fetch;
pub(crate) mod log;
pub(crate) mod pull;
pub(crate) mod push;
pub(crate) mod rebase;
pub(crate) mod reset;
pub(crate) mod show;
pub(crate) mod show_refs;
pub(crate) mod stage;
pub(crate) mod unstage;

pub(crate) type Action = Rc<dyn FnMut(&mut State, &mut Term) -> Res<()>>;

pub(crate) trait OpTrait: Display + PartialEq {
    /// Get the implementation (which may or may not exist) of the Op given some TargetData.
    /// This indirection allows Gitu to show a contextual menu of applicable actions.
    fn get_action(&self, target: Option<&TargetData>) -> Option<Action>;

    /// This indicates whether the Op is meant to read and
    /// act on TargetData. Those are listed differently in the help menu.
    fn is_target_op(&self) -> bool {
        false
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) enum SubmenuOp {
    Any,
    Branch,
    Commit,
    Fetch,
    Help,
    Log,
    #[default]
    None,
    Pull,
    Push,
    Rebase,
    Reset,
}

impl Display for SubmenuOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            SubmenuOp::Any => "Any",
            SubmenuOp::Branch => "Branch",
            SubmenuOp::Commit => "Commit",
            SubmenuOp::Fetch => "Fetch",
            SubmenuOp::Help => "Help",
            SubmenuOp::Log => "Log",
            SubmenuOp::None => "None",
            SubmenuOp::Pull => "Pull",
            SubmenuOp::Push => "Push",
            SubmenuOp::Rebase => "Rebase",
            SubmenuOp::Reset => "Reset",
        })
    }
}

pub(crate) fn cmd(input: Vec<u8>, command: fn() -> Command) -> Action {
    Rc::new(move |state, term| state.run_external_cmd(term, &input, command()))
}

pub(crate) fn cmd_arg(command: fn(&OsStr) -> Command, arg: OsString) -> Action {
    Rc::new(move |state, term| state.run_external_cmd(term, &[], command(&arg)))
}

pub(crate) fn subscreen_arg(command: fn(&OsStr) -> Command, arg: OsString) -> Action {
    Rc::new(move |state, term| state.issue_subscreen_command(term, command(&arg)))
}
