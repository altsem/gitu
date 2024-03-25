use tui_prompts::State as _;

use crate::{items::TargetData, prompt::PromptData, state::State, term::Term, ErrorBuffer, Res};
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
pub(crate) mod stash;
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
    MoveDown,
    MoveUp,
    MoveDownLine,
    MoveUpLine,
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
    Stash,
    StashApply,
    StashIndex,
    StashWorktree,
    StashKeepIndex,
    StashPop,
    StashDrop,

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

    Menu(Menu),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Menu {
    Branch,
    Commit,
    Fetch,
    Help,
    Log,
    Pull,
    Push,
    Rebase,
    Reset,
    Stash,
}

impl Op {
    pub fn implementation(self) -> Box<dyn OpTrait> {
        match self {
            Op::Quit => Box::new(editor::Quit),
            Op::Menu(menu) => Box::new(editor::Menu(menu)),
            Op::Refresh => Box::new(editor::Refresh),
            Op::ToggleSection => Box::new(editor::ToggleSection),
            Op::MoveDown => Box::new(editor::MoveDown),
            Op::MoveUp => Box::new(editor::MoveUp),
            Op::MoveDownLine => Box::new(editor::MoveDownLine),
            Op::MoveUpLine => Box::new(editor::MoveUpLine),
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
            Op::Stash => Box::new(stash::Stash),
            Op::StashApply => Box::new(stash::StashApply),
            Op::StashIndex => Box::new(stash::StashIndex),
            Op::StashWorktree => Box::new(stash::StashWorktree),
            Op::StashKeepIndex => Box::new(stash::StashKeepIndex),
            Op::StashPop => Box::new(stash::StashPop),
            Op::StashDrop => Box::new(stash::StashDrop),

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
        }
    }
}

impl Display for Menu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Menu::Branch => "Branch",
            Menu::Commit => "Commit",
            Menu::Fetch => "Fetch",
            Menu::Help => "Help",
            Menu::Log => "Log",
            Menu::Pull => "Pull",
            Menu::Push => "Push",
            Menu::Rebase => "Rebase",
            Menu::Reset => "Reset",
            Menu::Stash => "Stash",
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

pub(crate) fn create_y_n_prompt(mut action: Action, prompt: &'static str) -> Action {
    let update_fn = Rc::new(move |state: &mut State, term: &mut Term| {
        if state.prompt.state.status().is_pending() {
            match state.prompt.state.value() {
                "y" => {
                    Rc::get_mut(&mut action).unwrap()(state, term)?;
                    state.prompt.reset(term)?;
                }
                "" => (),
                _ => {
                    state.error_buffer = Some(ErrorBuffer("Aborted".to_string()));
                    state.prompt.reset(term)?;
                }
            }
        }
        Ok(())
    });

    Rc::new(move |state: &mut State, _term: &mut Term| {
        state.prompt.set(PromptData {
            prompt_text: format!("{} (y or n)", prompt).into(),
            update_fn: update_fn.clone(),
        });

        Ok(())
    })
}
