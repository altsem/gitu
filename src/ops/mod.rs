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

pub(crate) trait OpTrait: Display {
    /// Get the implementation (which may or may not exist) of the Op given some TargetData.
    /// This indirection allows Gitu to show a contextual menu of applicable actions.
    fn get_action(&self, target: Option<&TargetData>) -> Option<Action>;

    /// This indicates whether the Op is meant to read and
    /// act on TargetData. Those are listed differently in the help menu.
    fn is_target_op(&self) -> bool {
        false
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Op {
    Quit,
    Refresh,

    ToggleSection,
    SelectNext,
    SelectPrevious,
    HalfPageUp,
    HalfPageDown,

    Checkout,
    CheckoutNewBranch,
    Commit,
    CommitAmend,
    FetchAll,
    LogCurrent,
    Pull,
    Push,
    RebaseAbort,
    RebaseContinue,
    ShowRefs,

    CommitFixup,
    Discard,
    LogOther,
    RebaseAutosquash,
    RebaseInteractive,
    ResetSoft,
    ResetMixed,
    ResetHard,
    Show,
    Stage,
    Unstage,

    Submenu(SubmenuOp),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum SubmenuOp {
    Any,
    Branch,
    Commit,
    Fetch,
    Help,
    Log,
    None,
    Pull,
    Push,
    Rebase,
    Reset,
}

impl Op {
    pub fn implementation(self) -> Box<dyn OpTrait> {
        match self {
            Op::Quit => Box::new(editor::Quit),
            Op::Refresh => Box::new(editor::Refresh),
            Op::ToggleSection => Box::new(editor::ToggleSection),
            Op::SelectNext => Box::new(editor::SelectNext),
            Op::SelectPrevious => Box::new(editor::SelectPrevious),
            Op::HalfPageUp => Box::new(editor::HalfPageUp),
            Op::HalfPageDown => Box::new(editor::HalfPageDown),

            Op::Checkout => Box::new(checkout::Checkout),
            Op::CheckoutNewBranch => Box::new(checkout::CheckoutNewBranch),
            Op::Commit => Box::new(commit::Commit),
            Op::CommitAmend => Box::new(commit::CommitAmend),
            Op::FetchAll => Box::new(fetch::FetchAll),
            Op::LogCurrent => Box::new(log::LogCurrent),
            Op::Pull => Box::new(pull::Pull),
            Op::Push => Box::new(push::Push),
            Op::RebaseAbort => Box::new(rebase::RebaseAbort),
            Op::RebaseContinue => Box::new(rebase::RebaseContinue),
            Op::ShowRefs => Box::new(show_refs::ShowRefs),

            Op::CommitFixup => Box::new(commit::CommitFixup),
            Op::Discard => Box::new(discard::Discard),
            Op::LogOther => Box::new(log::LogOther),
            Op::RebaseAutosquash => Box::new(rebase::RebaseAutosquash),
            Op::RebaseInteractive => Box::new(rebase::RebaseInteractive),
            Op::ResetSoft => Box::new(reset::ResetSoft),
            Op::ResetMixed => Box::new(reset::ResetMixed),
            Op::ResetHard => Box::new(reset::ResetHard),
            Op::Show => Box::new(show::Show),
            Op::Stage => Box::new(stage::Stage),
            Op::Unstage => Box::new(unstage::Unstage),

            op => unimplemented!("{:?}", op),
        }
    }
}

impl OpTrait for Op {
    fn get_action(&self, target: Option<&TargetData>) -> Option<Action> {
        self.implementation().get_action(target)
    }

    fn is_target_op(&self) -> bool {
        match self {
            // TODO get rid of this special case
            Op::Submenu(_) => false,
            _ => self.implementation().is_target_op(),
        }
    }
}

impl Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.implementation().fmt(f)
    }
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
