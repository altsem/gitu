use serde::{Deserialize, Serialize};

use crate::{
    Res,
    app::{App, State},
    item_data::ItemData,
    menu::Menu,
    term::Term,
};
use std::{fmt::Display, rc::Rc};

pub(crate) mod branch;
pub(crate) mod commit;
pub(crate) mod copy_hash;
pub(crate) mod discard;
pub(crate) mod editor;
pub(crate) mod fetch;
pub(crate) mod log;
pub(crate) mod merge;
pub(crate) mod pull;
pub(crate) mod push;
pub(crate) mod rebase;
pub(crate) mod remote;
pub(crate) mod reset;
pub(crate) mod revert;
pub(crate) mod show;
pub(crate) mod show_refs;
pub(crate) mod stage;
pub(crate) mod stash;
pub(crate) mod unstage;

pub(crate) type Action = Rc<dyn FnMut(&mut App, &mut Term) -> Res<()>>;

pub(crate) trait OpTrait {
    /// Get the implementation (which may or may not exist) of the Op given some TargetData.
    /// This indirection allows Gitu to show a contextual menu of applicable actions.
    fn get_action(&self, target: &ItemData) -> Option<Action>;

    /// This indicates whether the Op is meant to read and
    /// act on TargetData. Those are listed differently in the help menu.
    fn is_target_op(&self) -> bool {
        false
    }

    fn display(&self, state: &State) -> String;
}

#[derive(Clone, PartialOrd, Ord, PartialEq, Eq, Debug, Serialize, Deserialize, strum::AsRefStr)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub(crate) enum Op {
    AddRemote,
    Checkout,
    CheckoutNewBranch,
    Spinoff,
    Delete,
    Commit,
    CommitAmend,
    CommitExtend,
    FetchAll,
    FetchPushRemote,
    FetchUpstream,
    FetchElsewhere,
    LogCurrent,
    PullFromPushRemote,
    PullFromUpstream,
    PullFromElsewhere,
    PushToPushRemote,
    PushToUpstream,
    PushToElsewhere,
    RebaseAbort,
    RebaseContinue,
    RebaseElsewhere,
    RemoveRemote,
    RenameRemote,
    ShowRefs,
    Stash,
    StashApply,
    StashIndex,
    StashWorktree,
    StashKeepIndex,
    StashPop,
    StashDrop,
    CommitFixup,
    CommitInstantFixup,
    LogOther,
    RebaseAutosquash,
    RebaseInteractive,
    ResetSoft,
    ResetMixed,
    ResetHard,
    RevertAbort,
    RevertContinue,
    RevertCommit,
    Merge,
    MergeAbort,
    MergeContinue,

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

    Refresh,
    Quit,

    #[serde(untagged)]
    OpenMenu(Menu),
    #[serde(untagged)]
    ToggleArg(String),
    #[serde(untagged)]
    MoveToScreenLine(usize),
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
            Op::MoveToScreenLine(screen_line) => Box::new(editor::MoveToScreenLine(screen_line)),
            Op::MoveNextSection => Box::new(editor::MoveNextSection),
            Op::MovePrevSection => Box::new(editor::MovePrevSection),
            Op::MoveParentSection => Box::new(editor::MoveParentSection),
            Op::HalfPageUp => Box::new(editor::HalfPageUp),
            Op::HalfPageDown => Box::new(editor::HalfPageDown),
            Op::Checkout => Box::new(branch::Checkout),
            Op::CheckoutNewBranch => Box::new(branch::CheckoutNewBranch),
            Op::Spinoff => Box::new(branch::Spinoff),
            Op::Delete => Box::new(branch::Delete),
            Op::Commit => Box::new(commit::Commit),
            Op::CommitAmend => Box::new(commit::CommitAmend),
            Op::CommitExtend => Box::new(commit::CommitExtend),
            Op::FetchAll => Box::new(fetch::FetchAll),
            Op::FetchElsewhere => Box::new(fetch::FetchElsewhere),
            Op::FetchPushRemote => Box::new(fetch::FetchPushRemote),
            Op::FetchUpstream => Box::new(fetch::FetchUpstream),
            Op::LogCurrent => Box::new(log::LogCurrent),
            Op::PullFromPushRemote => Box::new(pull::PullFromPushRemote),
            Op::PullFromUpstream => Box::new(pull::PullFromUpstream),
            Op::PullFromElsewhere => Box::new(pull::PullFromElsewhere),
            Op::PushToPushRemote => Box::new(push::PushToPushRemote),
            Op::PushToUpstream => Box::new(push::PushToUpstream),
            Op::PushToElsewhere => Box::new(push::PushToElsewhere),
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
            Op::CommitInstantFixup => Box::new(commit::CommitInstantFixup),
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

            Op::AddRemote => Box::new(remote::AddRemote),
            Op::RemoveRemote => Box::new(remote::RemoveRemote),
            Op::RenameRemote => Box::new(remote::RenameRemote),

            Op::Merge => Box::new(merge::Merge),
            Op::MergeAbort => Box::new(merge::MergeAbort),
            Op::MergeContinue => Box::new(merge::MergeContinue),
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
            Menu::Merge => "Merge",
            Menu::Remote => "Remote",
            Menu::Pull => "Pull",
            Menu::Push => "Push",
            Menu::Rebase => "Rebase",
            Menu::Reset => "Reset",
            Menu::Revert => "Revert",
            Menu::Stash => "Stash",
        })
    }
}

pub(crate) fn confirm(app: &mut App, term: &mut Term, prompt: &'static str) -> Res<()> {
    app.confirm(term, prompt)
}

pub(crate) fn selected_rev(state: &App) -> Option<String> {
    state.selected_rev()
}
