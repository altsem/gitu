use crate::{state::State, Res};
use ratatui::{backend::Backend, prelude::Terminal};
use std::{borrow::Cow, fmt::Display};
use strum::EnumIter;
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
pub(crate) mod show_refs;

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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Op {
    Quit,
    Refresh,

    ToggleSection(editor::ToggleSection),
    SelectNext(editor::SelectNext),
    SelectPrevious(editor::SelectPrevious),
    HalfPageUp(editor::HalfPageUp),
    HalfPageDown(editor::HalfPageDown),

    Checkout(checkout::Checkout),
    CheckoutNewBranch(checkout::CheckoutNewBranch),
    Commit(commit::Commit),
    CommitAmend(commit::CommitAmend),
    FetchAll(fetch::FetchAll),
    LogCurrent(log::LogCurrent),
    Pull(pull::Pull),
    Push(push::Push),
    RebaseAbort(rebase::RebaseAbort),
    RebaseContinue(rebase::RebaseContinue),
    ShowRefs(show_refs::ShowRefs),

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

#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumIter)]
pub(crate) enum TargetOp {
    CommitFixup,
    Discard(discard::Discard),
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
        match self {
            Op::ToggleSection(op_trait) => Box::new(op_trait),
            Op::SelectNext(op_trait) => Box::new(op_trait),
            Op::SelectPrevious(op_trait) => Box::new(op_trait),
            Op::HalfPageUp(op_trait) => Box::new(op_trait),
            Op::HalfPageDown(op_trait) => Box::new(op_trait),

            Op::Checkout(op_trait) => Box::new(op_trait),
            Op::CheckoutNewBranch(op_trait) => Box::new(op_trait),
            Op::Commit(op_trait) => Box::new(op_trait),
            Op::CommitAmend(op_trait) => Box::new(op_trait),
            Op::FetchAll(op_trait) => Box::new(op_trait),
            Op::LogCurrent(op_trait) => Box::new(op_trait),
            Op::Pull(op_trait) => Box::new(op_trait),
            Op::Push(op_trait) => Box::new(op_trait),
            Op::RebaseAbort(op_trait) => Box::new(op_trait),
            Op::RebaseContinue(op_trait) => Box::new(op_trait),
            Op::ShowRefs(op_trait) => Box::new(op_trait),
            Op::Target(TargetOp::Discard(op_trait)) => Box::new(op_trait),
            _ => unimplemented!(),
        }
    }
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

impl Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Op::Checkout(_) => "Checkout branch/revision",
            Op::CheckoutNewBranch(_) => "Checkout new branch",
            Op::Commit(_) => "Commit",
            Op::CommitAmend(_) => "Commit amend",
            Op::FetchAll(_) => "Fetch all",
            Op::HalfPageDown(_) => "Half page down",
            Op::HalfPageUp(_) => "Half page up",
            Op::LogCurrent(_) => "Log current",
            Op::Pull(_) => "Pull",
            Op::Push(_) => "Push",
            Op::Quit => "Quit",
            Op::RebaseAbort(_) => "Rebase abort",
            Op::RebaseContinue(_) => "Rebase continue",
            Op::Refresh => "Refresh",
            Op::SelectNext(_) => "Select next",
            Op::SelectPrevious(_) => "Select previous",
            Op::ShowRefs(_) => "Show refs",
            Op::Submenu(_) => "Submenu",
            Op::Target(_) => "Target",
            Op::ToggleSection(_) => "Toggle section",
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
        f.write_str(match self {
            TargetOp::CommitFixup => "Commit fixup",
            TargetOp::Discard(_) => "Discard",
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
