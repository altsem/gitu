use crate::{items::TargetData, state::State, term::Term, Res};
use std::{
    borrow::Cow,
    ffi::{OsStr, OsString},
    fmt::Display,
    process::Command,
};
use tui_prompts::prelude::Status;

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

pub(crate) trait OpTrait: Display {
    fn trigger(&self, state: &mut State, term: &mut Term) -> Res<()>;

    fn format_prompt(&self, _state: &State) -> Cow<'static, str> {
        unimplemented!()
    }

    fn prompt_update(&self, _status: Status, _state: &mut State, _term: &mut Term) -> Res<()> {
        unimplemented!()
    }
}

pub(crate) type Action = Box<dyn FnMut(&mut State, &mut Term) -> Res<()>>;

pub(crate) trait TargetOpTrait: Display {
    fn get_action(&self, target: TargetData) -> Option<Action>;
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

    Submenu(SubmenuOp),
    Target(TargetOp),
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum TargetOp {
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
            Op::Target(TargetOp::Discard) => Box::new(discard::Discard),
            op => unimplemented!("{:?}", op),
        }
    }
}

impl TargetOp {
    pub fn implementation(self) -> Box<dyn TargetOpTrait> {
        match self {
            TargetOp::CommitFixup => Box::new(commit::CommitFixup),
            TargetOp::Discard => Box::new(discard::Discard),
            TargetOp::LogOther => Box::new(log::LogOther),
            TargetOp::RebaseAutosquash => Box::new(rebase::RebaseAutosquash),
            TargetOp::RebaseInteractive => Box::new(rebase::RebaseInteractive),
            TargetOp::ResetSoft => Box::new(reset::ResetSoft),
            TargetOp::ResetMixed => Box::new(reset::ResetMixed),
            TargetOp::ResetHard => Box::new(reset::ResetHard),
            TargetOp::Show => Box::new(show::Show),
            TargetOp::Stage => Box::new(stage::Stage),
            TargetOp::Unstage => Box::new(unstage::Unstage),
        }
    }
}

pub(crate) fn get_action(target_data: Option<TargetData>, target_op: TargetOp) -> Option<Action> {
    target_data.and_then(|data| TargetOpTrait::get_action(&target_op, data))
}

impl OpTrait for Op {
    fn trigger(&self, state: &mut State, term: &mut Term) -> Res<()> {
        self.implementation().trigger(state, term)?;
        Ok(())
    }

    fn format_prompt(&self, state: &State) -> Cow<'static, str> {
        self.implementation().format_prompt(state)
    }

    fn prompt_update(&self, status: Status, arg: &mut State, term: &mut Term) -> Res<()> {
        self.implementation().prompt_update(status, arg, term)?;
        Ok(())
    }
}

impl TargetOpTrait for TargetOp {
    fn get_action(&self, target: TargetData) -> Option<Action> {
        self.implementation().get_action(target)
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

impl Display for TargetOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.implementation().fmt(f)
    }
}

pub(crate) fn cmd(input: Vec<u8>, command: fn() -> Command) -> Option<Action> {
    Some(Box::new(move |state, term| {
        state.run_external_cmd(term, &input, command())
    }))
}

pub(crate) fn cmd_arg(command: fn(&OsStr) -> Command, arg: OsString) -> Option<Action> {
    Some(Box::new(move |state, term| {
        state.run_external_cmd(term, &[], command(&arg))
    }))
}

pub(crate) fn subscreen_arg(command: fn(&OsStr) -> Command, arg: OsString) -> Option<Action> {
    Some(Box::new(move |state, term| {
        state.issue_subscreen_command(term, command(&arg))
    }))
}
