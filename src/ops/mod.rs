use crate::{state::State, Res};
use ratatui::{backend::Backend, prelude::Terminal};
use std::borrow::Cow;
use strum::EnumIter;
use tui_prompts::prelude::Status;

pub(crate) mod checkout;
pub(crate) mod commit;
pub(crate) mod discard;
pub(crate) mod fetch;
pub(crate) mod log;
pub(crate) mod pull;
pub(crate) mod push;
pub(crate) mod rebase;
pub(crate) mod show_refs;

pub(crate) trait OpTrait<B: Backend> {
    fn trigger(&self, state: &mut State, term: &mut Terminal<B>) -> Res<()>;

    fn format_prompt(&self) -> Cow<'static, str> {
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
    CheckoutNewBranch,
    Commit,
    CommitAmend,
    FetchAll,
    HalfPageDown,
    HalfPageUp,
    LogCurrent,
    Pull,
    Push,
    Quit,
    RebaseAbort,
    RebaseContinue,
    Refresh,
    SelectNext,
    SelectPrevious,
    ShowRefs,
    Submenu(SubmenuOp),
    Target(TargetOp),
    ToggleSection,
}

impl Op {
    pub fn implementation<B: Backend>(&self) -> Box<dyn OpTrait<B>> {
        match self {
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

impl<B: Backend> OpTrait<B> for Op {
    fn trigger(&self, state: &mut State, term: &mut Terminal<B>) -> Res<()> {
        self.implementation::<B>().trigger(state, term)?;
        Ok(())
    }

    fn format_prompt(&self) -> Cow<'static, str> {
        self.implementation::<B>().format_prompt()
    }

    fn prompt_update(
        &self,
        status: tui_prompts::prelude::Status,
        arg: &mut State,
        term: &mut ratatui::prelude::Terminal<B>,
    ) -> Res<()> {
        self.implementation::<B>()
            .prompt_update(status, arg, term)?;
        Ok(())
    }
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
    Checkout,
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
