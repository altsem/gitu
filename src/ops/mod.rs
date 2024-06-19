use serde::{Deserialize, Serialize};
use tui_prompts::State as _;

use crate::{
    cmd_log::CmdLogEntry, items::TargetData, menu::Menu, prompt::PromptData, state::State,
    term::Term, Res,
};
use std::{fmt::Display, rc::Rc};

pub(crate) mod checkout;
pub(crate) mod commit;
pub(crate) mod copy_hash;
pub(crate) mod discard;
pub(crate) mod editor;
pub(crate) mod fetch;
pub(crate) mod log;
pub(crate) mod pull;
pub(crate) mod push;
pub(crate) mod rebase;
pub(crate) mod reset;
pub(crate) mod revert;
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

#[derive(Clone, PartialOrd, Ord, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Op {
    Checkout,
    CheckoutNewBranch,
    Commit,
    CommitAmend,
    FetchAll,
    FetchElsewhere,
    LogCurrent,
    Pull,
    PullElsewhere,
    Push,
    PushElsewhere,
    RebaseAbort,
    RebaseContinue,
    RebaseElsewhere,
    ShowRefs,
    Stash,
    StashApply,
    StashIndex,
    StashWorktree,
    StashKeepIndex,
    StashPop,
    StashDrop,
    CommitFixup,
    LogOther,
    RebaseAutosquash,
    RebaseInteractive,
    ResetSoft,
    ResetMixed,
    ResetHard,
    RevertAbort,
    RevertContinue,
    RevertCommit,

    Stage,
    Unstage,
    Show,
    Discard,
    CopyHash,

    ToggleSection,
    MoveUp,
    MoveDown,
    MoveUpLine,
    MoveDownLine,
    MovePrevSection,
    MoveNextSection,
    MoveParentSection,
    HalfPageUp,
    HalfPageDown,
    ScrollDown,
    ScrollUp,

    Refresh,
    Quit,

    #[serde(untagged)]
    OpenMenu(Menu),
    #[serde(untagged)]
    ToggleArg(String),
}

impl Op {
    pub fn implementation(self) -> Box<dyn OpTrait> {
        match self {
            Op::Quit => Box::new(editor::Quit),
            Op::OpenMenu(menu) => Box::new(editor::OpenMenu(menu)),
            Op::Refresh => Box::new(editor::Refresh),
            Op::ToggleArg(name) => Box::new(editor::ToggleArg(name)),
            Op::ToggleSection => Box::new(editor::ToggleSection),
            Op::MoveDown => Box::new(editor::MoveDown),
            Op::MoveUp => Box::new(editor::MoveUp),
            Op::MoveDownLine => Box::new(editor::MoveDownLine),
            Op::MoveUpLine => Box::new(editor::MoveUpLine),
            Op::MoveNextSection => Box::new(editor::MoveNextSection),
            Op::MovePrevSection => Box::new(editor::MovePrevSection),
            Op::MoveParentSection => Box::new(editor::MoveParentSection),
            Op::HalfPageUp => Box::new(editor::HalfPageUp),
            Op::HalfPageDown => Box::new(editor::HalfPageDown),
            Op::ScrollDown => Box::new(editor::ScrollDown),
            Op::ScrollUp => Box::new(editor::ScrollUp),

            Op::Checkout => Box::new(checkout::Checkout),
            Op::CheckoutNewBranch => Box::new(checkout::CheckoutNewBranch),
            Op::Commit => Box::new(commit::Commit),
            Op::CommitAmend => Box::new(commit::CommitAmend),
            Op::FetchAll => Box::new(fetch::FetchAll),
            Op::FetchElsewhere => Box::new(fetch::FetchElsewhere),
            Op::LogCurrent => Box::new(log::LogCurrent),
            Op::Pull => Box::new(pull::Pull),
            Op::PullElsewhere => Box::new(pull::PullElsewhere),
            Op::Push => Box::new(push::Push),
            Op::PushElsewhere => Box::new(push::PushElsewhere),
            Op::RebaseAbort => Box::new(rebase::RebaseAbort),
            Op::RebaseContinue => Box::new(rebase::RebaseContinue),
            Op::RebaseElsewhere => Box::new(rebase::RebaseElsewhere),
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
            Op::RevertAbort => Box::new(revert::RevertAbort),
            Op::RevertContinue => Box::new(revert::RevertContinue),
            Op::RevertCommit => Box::new(revert::RevertCommit),
            Op::Show => Box::new(show::Show),
            Op::Stage => Box::new(stage::Stage),
            Op::Unstage => Box::new(unstage::Unstage),
            Op::CopyHash => Box::new(copy_hash::CopyHash),
        }
    }
}

impl Display for Menu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Menu::Root => "Root",
            Menu::Branch => "Branch",
            Menu::Commit => "Commit",
            Menu::Fetch => "Fetch",
            Menu::Help => "Help",
            Menu::Log => "Log",
            Menu::Pull => "Pull",
            Menu::Push => "Push",
            Menu::Rebase => "Rebase",
            Menu::Reset => "Reset",
            Menu::Revert => "Revert",
            Menu::Stash => "Stash",
        })
    }
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
                    state
                        .current_cmd_log
                        .push(CmdLogEntry::Error("Aborted".to_string()));
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

pub(crate) fn create_prompt(
    prompt: &'static str,
    callback: fn(&mut State, &mut Term, &str) -> Res<()>,
    hide_menu: bool,
) -> Action {
    create_prompt_with_default(prompt, callback, |_| None, hide_menu)
}

pub(crate) fn create_prompt_with_default(
    prompt: &'static str,
    callback: fn(&mut State, &mut Term, &str) -> Res<()>,
    default_fn: fn(&State) -> Option<String>,
    hide_menu: bool,
) -> Action {
    Rc::new(move |state: &mut State, _term: &mut Term| {
        set_prompt(
            state,
            prompt,
            invoke_default,
            Box::new(default_fn),
            callback,
            hide_menu,
        );
        Ok(())
    })
}

fn invoke_default(
    state: &mut State,
    term: &mut Term,
    value: &str,
    context: &fn(&mut State, &mut Term, &str) -> Res<()>,
) -> Res<()> {
    context(state, term, value)
}

type DefaultFn = Box<dyn Fn(&State) -> Option<String>>;

pub(crate) fn set_prompt<T: 'static>(
    state: &mut State,
    prompt: &'static str,
    callback: fn(&mut State, &mut Term, &str, &T) -> Res<()>,
    default_fn: DefaultFn,
    context: T,
    hide_menu: bool,
) {
    let prompt_text = if let Some(default) = default_fn(state) {
        format!("{} (default {}):", prompt, default).into()
    } else {
        format!("{}:", prompt).into()
    };

    if hide_menu {
        state.hide_menu();
    }

    state.prompt.set(PromptData {
        prompt_text,
        update_fn: Rc::new(move |state, term| {
            if state.prompt.state.status().is_done() {
                let input = state.prompt.state.value().to_string();
                state.prompt.reset(term)?;

                let default_value = default_fn(state);
                let value = match (input.as_str(), &default_value) {
                    ("", None) => "",
                    ("", Some(selected)) => selected,
                    (value, _) => value,
                };

                callback(state, term, value, &context)?;

                if hide_menu {
                    state.unhide_menu();
                }
            }
            Ok(())
        }),
    });
}

pub(crate) fn selected_rev(state: &State) -> Option<String> {
    match &state.screen().get_selected_item().target_data {
        Some(TargetData::Branch(branch)) => Some(branch.to_owned()),
        Some(TargetData::Commit(commit)) => Some(commit.to_owned()),
        _ => None,
    }
}
