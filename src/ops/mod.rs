use crate::{items::TargetData, state::State, Res};
use ratatui::{backend::Backend, prelude::Terminal};
use std::{
    borrow::Cow,
    ffi::{OsStr, OsString},
    fmt::Display,
    process::Command,
};
use strum::{EnumIter, IntoEnumIterator};
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
    CommitFixup(commit::CommitFixup),
    Discard(discard::Discard),
    LogOther(log::LogOther),
    RebaseAutosquash(rebase::RebaseAutosquash),
    RebaseInteractive(rebase::RebaseInteractive),
    ResetSoft(reset::ResetSoft),
    ResetMixed(reset::ResetMixed),
    ResetHard(reset::ResetHard),
    Show(show::Show),
    Stage(stage::Stage),
    Unstage(unstage::Unstage),
}

impl Op {
    pub fn implementation<B: Backend>(self) -> Box<dyn OpTrait<B>> {
        // TODO Get rid of this
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

impl TargetOp {
    pub fn implementation<B: Backend>(self) -> Box<dyn TargetOpTrait<B>> {
        // TODO Get rid of this
        match self {
            TargetOp::CommitFixup(op_trait) => Box::new(op_trait),
            TargetOp::Discard(op_trait) => Box::new(op_trait),
            TargetOp::LogOther(op_trait) => Box::new(op_trait),
            TargetOp::RebaseAutosquash(op_trait) => Box::new(op_trait),
            TargetOp::RebaseInteractive(op_trait) => Box::new(op_trait),
            TargetOp::ResetSoft(op_trait) => Box::new(op_trait),
            TargetOp::ResetMixed(op_trait) => Box::new(op_trait),
            TargetOp::ResetHard(op_trait) => Box::new(op_trait),
            TargetOp::Show(op_trait) => Box::new(op_trait),
            TargetOp::Stage(op_trait) => Box::new(op_trait),
            TargetOp::Unstage(op_trait) => Box::new(op_trait),
        }
    }
}

pub(crate) fn get_action<B: Backend>(
    target_data: Option<TargetData>,
    target_op: TargetOp,
) -> Option<Action<B>> {
    target_data.and_then(|data| TargetOpTrait::<B>::get_action(&target_op, data))
}

pub(crate) fn list_target_ops<B: Backend>(
    data: &TargetData,
) -> impl Iterator<Item = (TargetOp, TargetData)> + '_ {
    TargetOp::iter()
        .filter(|target_op| TargetOpTrait::<B>::get_action(target_op, data.clone()).is_some())
        .map(|op| (op, data.clone()))
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
        // TODO Move this to each module
        f.write_str(match self {
            TargetOp::CommitFixup(_) => "Commit fixup",
            TargetOp::Discard(_) => "Discard",
            TargetOp::LogOther(_) => "Log other",
            TargetOp::RebaseAutosquash(_) => "Rebase autosquash",
            TargetOp::RebaseInteractive(_) => "Rebase interactive",
            TargetOp::ResetSoft(_) => "Reset soft",
            TargetOp::ResetMixed(_) => "Reset mixed",
            TargetOp::ResetHard(_) => "Reset hard",
            TargetOp::Show(_) => "Show",
            TargetOp::Stage(_) => "Stage",
            TargetOp::Unstage(_) => "Unstage",
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
