use crossterm::event::{self, KeyCode, KeyModifiers};

use KeyCode::*;
use Op::*;
use TargetOp::*;
use TransientOp::*;

pub(crate) struct Keybind {
    pub transient: TransientOp,
    pub mods: KeyModifiers,
    pub key: KeyCode,
    pub op: Op,
}

impl Keybind {
    const fn nomod(transient: TransientOp, key: KeyCode, op: Op) -> Self {
        Self {
            transient,
            mods: KeyModifiers::NONE,
            key,
            op,
        }
    }

    const fn ctrl(transient: TransientOp, key: KeyCode, op: Op) -> Self {
        Self {
            transient,
            mods: KeyModifiers::CONTROL,
            key,
            op,
        }
    }

    const fn shift(transient: TransientOp, key: KeyCode, op: Op) -> Self {
        Self {
            transient,
            mods: KeyModifiers::SHIFT,
            key,
            op,
        }
    }

    pub(crate) fn format_key(&self) -> String {
        match self.key {
            KeyCode::Enter => "ret".to_string(),
            KeyCode::Left => "←".to_string(),
            KeyCode::Right => "→".to_string(),
            KeyCode::Up => "↑".to_string(),
            KeyCode::Down => "↓".to_string(),
            KeyCode::Tab => "tab".to_string(),
            KeyCode::Delete => "del".to_string(),
            KeyCode::Insert => "ins".to_string(),
            KeyCode::F(n) => format!("F{}", n),
            KeyCode::Char(c) => if self.mods.contains(KeyModifiers::SHIFT) {
                c.to_ascii_uppercase()
            } else {
                c
            }
            .to_string(),
            KeyCode::Esc => "esc".to_string(),
            _ => "???".to_string(),
        }
    }
}

pub(crate) const KEYBINDS: &[Keybind] = &[
    // Generic
    Keybind::nomod(Any, Char('q'), Quit),
    Keybind::nomod(Any, Esc, Quit),
    Keybind::nomod(None, Char('g'), Refresh),
    Keybind::nomod(None, Tab, ToggleSection),
    // Navigation
    Keybind::nomod(None, Char('k'), SelectPrevious),
    Keybind::nomod(None, Char('j'), SelectNext),
    Keybind::ctrl(None, Char('u'), HalfPageUp),
    Keybind::ctrl(None, Char('d'), HalfPageDown),
    // Help
    Keybind::nomod(None, Char('h'), Transient(Help)),
    // Branch
    Keybind::nomod(None, Char('b'), Transient(Branch)),
    Keybind::nomod(Branch, Char('b'), Target(Checkout)),
    // Commit
    Keybind::nomod(None, Char('c'), Transient(TransientOp::Commit)),
    Keybind::nomod(TransientOp::Commit, Char('c'), Op::Commit),
    Keybind::nomod(TransientOp::Commit, Char('a'), CommitAmend),
    Keybind::nomod(TransientOp::Commit, Char('f'), Target(CommitFixup)),
    // Fetch
    Keybind::nomod(None, Char('f'), Transient(Fetch)),
    Keybind::nomod(Fetch, Char('a'), FetchAll),
    // Log
    Keybind::nomod(None, Char('l'), Transient(Log)),
    Keybind::nomod(Log, Char('l'), LogCurrent),
    // Pull
    Keybind::shift(None, Char('F'), Transient(Pull)),
    Keybind::nomod(Pull, Char('p'), PullRemote),
    // Push
    Keybind::shift(None, Char('P'), Transient(Push)),
    Keybind::nomod(Push, Char('p'), PushRemote),
    // Rebase
    Keybind::nomod(None, Char('r'), Transient(Rebase)),
    Keybind::nomod(Rebase, Char('i'), Target(RebaseInteractive)),
    Keybind::nomod(Rebase, Char('a'), RebaseAbort),
    Keybind::nomod(Rebase, Char('c'), RebaseContinue),
    Keybind::nomod(Rebase, Char('f'), Target(RebaseAutosquash)),
    // Show refs
    Keybind::nomod(None, Char('y'), ShowRefs),
    // Discard
    Keybind::shift(None, Char('K'), Transient(TransientOp::Discard)),
    Keybind::nomod(TransientOp::Discard, Char('y'), Target(TargetOp::Discard)),
    Keybind::nomod(TransientOp::Discard, Char('n'), Quit),
    // Target actions
    Keybind::nomod(None, Enter, Target(Show)),
    Keybind::nomod(None, Char('s'), Target(Stage)),
    Keybind::nomod(None, Char('u'), Target(Unstage)),
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Op {
    Quit,
    Refresh,
    SelectPrevious,
    SelectNext,
    ToggleSection,
    HalfPageUp,
    HalfPageDown,
    PushRemote,
    PullRemote,
    Transient(TransientOp),
    Commit,
    CommitAmend,
    FetchAll,
    LogCurrent,
    RebaseAbort,
    RebaseContinue,
    ShowRefs,
    Target(TargetOp),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum TransientOp {
    Any,
    None,
    Branch,
    Commit,
    Discard,
    Fetch,
    Help,
    Log,
    Pull,
    Push,
    Rebase,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum TargetOp {
    Checkout,
    CommitFixup,
    Show,
    Stage,
    Unstage,
    RebaseAutosquash,
    RebaseInteractive,
    Discard,
}

impl TargetOp {
    pub(crate) fn list_all() -> impl Iterator<Item = &'static TargetOp> {
        [
            &TargetOp::Checkout,
            &TargetOp::CommitFixup,
            &TargetOp::Show,
            &TargetOp::Stage,
            &TargetOp::Unstage,
            &TargetOp::RebaseAutosquash,
            &TargetOp::RebaseInteractive,
            &TargetOp::Discard,
        ]
        .into_iter()
    }
}

pub(crate) fn op_of_key_event(pending: TransientOp, key: event::KeyEvent) -> Option<Op> {
    KEYBINDS
        .iter()
        .find(|keybind| {
            (keybind.transient, keybind.mods, keybind.key) == (pending, key.modifiers, key.code)
                || (keybind.transient, keybind.mods, keybind.key) == (Any, key.modifiers, key.code)
        })
        .map(|keybind| keybind.op)
}

pub(crate) fn list(pending: &TransientOp) -> impl Iterator<Item = &Keybind> {
    let expected = if pending == &Help { None } else { *pending };

    KEYBINDS
        .iter()
        .filter(move |keybind| keybind.transient == expected)
}
