use crate::{items::TargetData, state::State, Res};
use ratatui::{backend::Backend, prelude::Terminal};
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

pub(crate) trait OpTrait<B: Backend> {
    fn trigger(&self, state: &mut State, term: &mut Terminal<B>) -> Res<()>;

    fn format_prompt(&self, _state: &State) -> Cow<'static, str> {
        unimplemented!()
    }

    fn prompt_update(
        &self,
        _status: Status,
        _state: &mut State,
        _term: &mut Terminal<B>,
    ) -> Res<()> {
        unimplemented!()
    }
}

pub(crate) type Action<B> = Box<dyn FnMut(&mut State, &mut Terminal<B>) -> Res<()>>;

pub(crate) trait TargetOpTrait<B: Backend> {
    fn get_action(&self, target: TargetData) -> Option<Action<B>>;
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
    pub fn implementation<B: Backend>(self) -> Box<dyn OpTrait<B>> {
        // TODO Get rid of this
        match self {
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
            _ => unimplemented!(),
        }
    }
}

impl TargetOp {
    pub fn implementation<B: Backend>(self) -> Box<dyn TargetOpTrait<B>> {
        // TODO Get rid of this
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

pub(crate) fn get_action<B: Backend>(
    target_data: Option<TargetData>,
    target_op: TargetOp,
) -> Option<Action<B>> {
    target_data.and_then(|data| TargetOpTrait::<B>::get_action(&target_op, data))
}

impl<B: Backend> OpTrait<B> for Op {
    fn trigger(&self, state: &mut State, term: &mut Terminal<B>) -> Res<()> {
        self.implementation::<B>().trigger(state, term)?;
        Ok(())
    }

    fn format_prompt(&self, state: &State) -> Cow<'static, str> {
        self.implementation::<B>().format_prompt(state)
    }

    fn prompt_update(&self, status: Status, arg: &mut State, term: &mut Terminal<B>) -> Res<()> {
        self.implementation::<B>()
            .prompt_update(status, arg, term)?;
        Ok(())
    }
}

impl<B: Backend> TargetOpTrait<B> for TargetOp {
    fn get_action(&self, target: TargetData) -> Option<Action<B>> {
        self.implementation().get_action(target)
    }
}

impl Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO Move this to each module
        f.write_str(match self {
            Op::Checkout => "Checkout branch/revision",
            Op::CheckoutNewBranch => "Checkout new branch",
            Op::Commit => "Commit",
            Op::CommitAmend => "Commit amend",
            Op::FetchAll => "Fetch all",
            Op::HalfPageDown => "Half page down",
            Op::HalfPageUp => "Half page up",
            Op::LogCurrent => "Log current",
            Op::Pull => "Pull",
            Op::Push => "Push",
            Op::Quit => "Quit",
            Op::RebaseAbort => "Rebase abort",
            Op::RebaseContinue => "Rebase continue",
            Op::Refresh => "Refresh",
            Op::SelectNext => "Select next",
            Op::SelectPrevious => "Select previous",
            Op::ShowRefs => "Show refs",
            Op::Submenu(_) => "Submenu",
            Op::Target(_) => "Target",
            Op::ToggleSection => "Toggle section",
        })
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
        // TODO Move this to each module
        f.write_str(match self {
            TargetOp::CommitFixup => "Commit fixup",
            TargetOp::Discard => "Discard",
            TargetOp::LogOther => "Log other",
            TargetOp::RebaseAutosquash => "Rebase autosquash",
            TargetOp::RebaseInteractive => "Rebase interactive",
            TargetOp::ResetSoft => "Reset soft",
            TargetOp::ResetMixed => "Reset mixed",
            TargetOp::ResetHard => "Reset hard",
            TargetOp::Show => "Show",
            TargetOp::Stage => "Stage",
            TargetOp::Unstage => "Unstage",
        })
    }
}

pub(crate) fn cmd<B: Backend>(input: Vec<u8>, command: fn() -> Command) -> Option<Action<B>> {
    Some(Box::new(move |state, term| {
        state.run_external_cmd(term, &input, command())
    }))
}

pub(crate) fn cmd_arg<B: Backend>(
    command: fn(&OsStr) -> Command,
    arg: OsString,
) -> Option<Action<B>> {
    Some(Box::new(move |state, term| {
        state.run_external_cmd(term, &[], command(&arg))
    }))
}

pub(crate) fn subscreen_arg<B: Backend>(
    command: fn(&OsStr) -> Command,
    arg: OsString,
) -> Option<Action<B>> {
    Some(Box::new(move |state, term| {
        state.issue_subscreen_command(term, command(&arg))
    }))
}
