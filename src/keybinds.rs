use crate::ops::checkout::Checkout;
use crate::ops::checkout::CheckoutNewBranch;
use crate::ops::commit::Commit;
use crate::ops::commit::CommitAmend;
use crate::ops::discard::Discard;
use crate::ops::editor::HalfPageDown;
use crate::ops::editor::HalfPageUp;
use crate::ops::editor::SelectNext;
use crate::ops::editor::SelectPrevious;
use crate::ops::editor::ToggleSection;
use crate::ops::fetch::FetchAll;
use crate::ops::log::LogCurrent;
use crate::ops::pull::Pull;
use crate::ops::push::Push;
use crate::ops::rebase::RebaseAbort;
use crate::ops::rebase::RebaseContinue;
use crate::ops::show_refs::ShowRefs;
use crate::ops::Op;
use crate::ops::SubmenuOp;
use crate::ops::TargetOp;
use crossterm::event::{self, KeyCode, KeyModifiers};
use KeyCode::*;

pub(crate) struct Keybind {
    pub submenu: SubmenuOp,
    pub mods: KeyModifiers,
    pub key: KeyCode,
    pub op: Op,
}

impl Keybind {
    const fn nomod(submenu: SubmenuOp, key: KeyCode, op: Op) -> Self {
        Self {
            submenu,
            mods: KeyModifiers::NONE,
            key,
            op,
        }
    }

    const fn ctrl(submenu: SubmenuOp, key: KeyCode, op: Op) -> Self {
        Self {
            submenu,
            mods: KeyModifiers::CONTROL,
            key,
            op,
        }
    }

    const fn shift(submenu: SubmenuOp, key: KeyCode, op: Op) -> Self {
        Self {
            submenu,
            mods: KeyModifiers::SHIFT,
            key,
            op,
        }
    }

    pub(crate) fn format_key(&self) -> String {
        let modifiers = self
            .mods
            .iter_names()
            .map(|(name, _)| match name {
                "CONTROL" => "C-",
                "SHIFT" => "",
                _ => unimplemented!("format_key mod {}", name),
            })
            .collect::<String>();

        modifiers
            + &match self.key {
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
    Keybind::nomod(SubmenuOp::Any, Char('q'), Op::Quit),
    Keybind::nomod(SubmenuOp::Any, Esc, Op::Quit),
    Keybind::nomod(SubmenuOp::None, Char('g'), Op::Refresh),
    // Editor
    Keybind::nomod(SubmenuOp::None, Tab, Op::ToggleSection(ToggleSection)),
    Keybind::nomod(
        SubmenuOp::None,
        Char('k'),
        Op::SelectPrevious(SelectPrevious),
    ),
    Keybind::nomod(
        SubmenuOp::None,
        Char('p'),
        Op::SelectPrevious(SelectPrevious),
    ),
    Keybind::nomod(
        SubmenuOp::None,
        KeyCode::Up,
        Op::SelectPrevious(SelectPrevious),
    ),
    Keybind::nomod(SubmenuOp::None, Char('j'), Op::SelectNext(SelectNext)),
    Keybind::nomod(SubmenuOp::None, Char('n'), Op::SelectNext(SelectNext)),
    Keybind::nomod(SubmenuOp::None, KeyCode::Down, Op::SelectNext(SelectNext)),
    Keybind::ctrl(SubmenuOp::None, Char('u'), Op::HalfPageUp(HalfPageUp)),
    Keybind::ctrl(SubmenuOp::None, Char('d'), Op::HalfPageDown(HalfPageDown)),
    // Help
    Keybind::nomod(SubmenuOp::None, Char('h'), Op::Submenu(SubmenuOp::Help)),
    // Branch
    Keybind::nomod(SubmenuOp::None, Char('b'), Op::Submenu(SubmenuOp::Branch)),
    Keybind::nomod(SubmenuOp::Branch, Char('b'), Op::Checkout(Checkout)),
    Keybind::nomod(
        SubmenuOp::Branch,
        Char('c'),
        Op::CheckoutNewBranch(CheckoutNewBranch),
    ),
    // Commit
    Keybind::nomod(SubmenuOp::None, Char('c'), Op::Submenu(SubmenuOp::Commit)),
    Keybind::nomod(SubmenuOp::Commit, Char('c'), Op::Commit(Commit)),
    Keybind::nomod(SubmenuOp::Commit, Char('a'), Op::CommitAmend(CommitAmend)),
    Keybind::nomod(
        SubmenuOp::Commit,
        Char('f'),
        Op::Target(TargetOp::CommitFixup),
    ),
    // Fetch
    Keybind::nomod(SubmenuOp::None, Char('f'), Op::Submenu(SubmenuOp::Fetch)),
    Keybind::nomod(SubmenuOp::Fetch, Char('a'), Op::FetchAll(FetchAll)),
    // Log
    Keybind::nomod(SubmenuOp::None, Char('l'), Op::Submenu(SubmenuOp::Log)),
    Keybind::nomod(SubmenuOp::Log, Char('l'), Op::LogCurrent(LogCurrent)),
    Keybind::nomod(SubmenuOp::Log, Char('o'), Op::Target(TargetOp::LogOther)),
    // Pull
    Keybind::shift(SubmenuOp::None, Char('F'), Op::Submenu(SubmenuOp::Pull)),
    Keybind::nomod(SubmenuOp::Pull, Char('p'), Op::Pull(Pull)),
    // Push
    Keybind::shift(SubmenuOp::None, Char('P'), Op::Submenu(SubmenuOp::Push)),
    Keybind::nomod(SubmenuOp::Push, Char('p'), Op::Push(Push)),
    // Rebase
    Keybind::nomod(SubmenuOp::None, Char('r'), Op::Submenu(SubmenuOp::Rebase)),
    Keybind::nomod(
        SubmenuOp::Rebase,
        Char('i'),
        Op::Target(TargetOp::RebaseInteractive),
    ),
    Keybind::nomod(SubmenuOp::Rebase, Char('a'), Op::RebaseAbort(RebaseAbort)),
    Keybind::nomod(
        SubmenuOp::Rebase,
        Char('c'),
        Op::RebaseContinue(RebaseContinue),
    ),
    Keybind::nomod(
        SubmenuOp::Rebase,
        Char('f'),
        Op::Target(TargetOp::RebaseAutosquash),
    ),
    // Reset
    Keybind::shift(SubmenuOp::None, Char('X'), Op::Submenu(SubmenuOp::Reset)),
    Keybind::nomod(SubmenuOp::Reset, Char('s'), Op::Target(TargetOp::ResetSoft)),
    Keybind::nomod(
        SubmenuOp::Reset,
        Char('m'),
        Op::Target(TargetOp::ResetMixed),
    ),
    Keybind::nomod(SubmenuOp::Reset, Char('h'), Op::Target(TargetOp::ResetHard)),
    // Show refs
    Keybind::nomod(SubmenuOp::None, Char('y'), Op::ShowRefs(ShowRefs)),
    // Discard
    Keybind::shift(
        SubmenuOp::None,
        Char('K'),
        Op::Target(TargetOp::Discard(Discard)),
    ),
    // Target actions
    Keybind::nomod(SubmenuOp::None, Enter, Op::Target(TargetOp::Show)),
    Keybind::nomod(SubmenuOp::None, Char('s'), Op::Target(TargetOp::Stage)),
    Keybind::nomod(SubmenuOp::None, Char('u'), Op::Target(TargetOp::Unstage)),
];

pub(crate) fn op_of_key_event(pending: SubmenuOp, key: event::KeyEvent) -> Option<Op> {
    KEYBINDS
        .iter()
        .find(|keybind| {
            (keybind.submenu, keybind.mods, keybind.key) == (pending, key.modifiers, key.code)
                || (keybind.submenu, keybind.mods, keybind.key)
                    == (SubmenuOp::Any, key.modifiers, key.code)
        })
        .map(|keybind| keybind.op)
}

pub(crate) fn list(pending: &SubmenuOp) -> impl Iterator<Item = &Keybind> {
    let expected = if pending == &SubmenuOp::Help {
        SubmenuOp::None
    } else {
        *pending
    };

    KEYBINDS
        .iter()
        .filter(move |keybind| keybind.submenu == expected)
}
