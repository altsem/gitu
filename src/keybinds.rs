use crate::ops;
use crate::ops::OpTrait;
use crate::ops::SubmenuOp;
use crossterm::event::{self, KeyCode, KeyModifiers};
use KeyCode::*;

pub(crate) struct Keybind {
    pub submenu: SubmenuOp,
    pub mods: KeyModifiers,
    pub key: KeyCode,
    pub op: &'static dyn OpTrait,
}

impl Keybind {
    const fn nomod(submenu: SubmenuOp, key: KeyCode, op: &'static dyn OpTrait) -> Self {
        Self {
            submenu,
            mods: KeyModifiers::NONE,
            key,
            op,
        }
    }

    const fn ctrl(submenu: SubmenuOp, key: KeyCode, op: &'static dyn OpTrait) -> Self {
        Self {
            submenu,
            mods: KeyModifiers::CONTROL,
            key,
            op,
        }
    }

    const fn shift(submenu: SubmenuOp, key: KeyCode, op: &'static dyn OpTrait) -> Self {
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
    Keybind::nomod(SubmenuOp::Any, Char('q'), &ops::editor::Quit),
    Keybind::nomod(SubmenuOp::Any, Esc, &ops::editor::Quit),
    Keybind::nomod(SubmenuOp::None, Char('g'), &ops::editor::Refresh),
    // Editor
    Keybind::nomod(SubmenuOp::None, Tab, &ops::editor::ToggleSection),
    Keybind::nomod(SubmenuOp::None, Char('k'), &ops::editor::SelectPrevious),
    Keybind::nomod(SubmenuOp::None, Char('p'), &ops::editor::SelectPrevious),
    Keybind::nomod(SubmenuOp::None, KeyCode::Up, &ops::editor::SelectPrevious),
    Keybind::nomod(SubmenuOp::None, Char('j'), &ops::editor::SelectNext),
    Keybind::nomod(SubmenuOp::None, Char('n'), &ops::editor::SelectNext),
    Keybind::nomod(SubmenuOp::None, KeyCode::Down, &ops::editor::SelectNext),
    Keybind::ctrl(SubmenuOp::None, Char('u'), &ops::editor::HalfPageUp),
    Keybind::ctrl(SubmenuOp::None, Char('d'), &ops::editor::HalfPageDown),
    // Help
    Keybind::nomod(
        SubmenuOp::None,
        Char('h'),
        &ops::editor::Submenu(SubmenuOp::Help),
    ),
    // Branch
    Keybind::nomod(
        SubmenuOp::None,
        Char('b'),
        &ops::editor::Submenu(SubmenuOp::Branch),
    ),
    Keybind::nomod(SubmenuOp::Branch, Char('b'), &ops::checkout::Checkout),
    Keybind::nomod(
        SubmenuOp::Branch,
        Char('c'),
        &ops::checkout::CheckoutNewBranch,
    ),
    // Commit
    Keybind::nomod(
        SubmenuOp::None,
        Char('c'),
        &ops::editor::Submenu(SubmenuOp::Commit),
    ),
    Keybind::nomod(SubmenuOp::Commit, Char('c'), &ops::commit::Commit),
    Keybind::nomod(SubmenuOp::Commit, Char('a'), &ops::commit::CommitAmend),
    Keybind::nomod(SubmenuOp::Commit, Char('f'), &ops::commit::CommitFixup),
    // Fetch
    Keybind::nomod(
        SubmenuOp::None,
        Char('f'),
        &ops::editor::Submenu(SubmenuOp::Fetch),
    ),
    Keybind::nomod(SubmenuOp::Fetch, Char('a'), &ops::fetch::FetchAll),
    // Log
    Keybind::nomod(
        SubmenuOp::None,
        Char('l'),
        &ops::editor::Submenu(SubmenuOp::Log),
    ),
    Keybind::nomod(SubmenuOp::Log, Char('l'), &ops::log::LogCurrent),
    Keybind::nomod(SubmenuOp::Log, Char('o'), &ops::log::LogOther),
    // Pull
    Keybind::shift(
        SubmenuOp::None,
        Char('F'),
        &ops::editor::Submenu(SubmenuOp::Pull),
    ),
    Keybind::nomod(SubmenuOp::Pull, Char('p'), &ops::pull::Pull),
    // Push
    Keybind::shift(
        SubmenuOp::None,
        Char('P'),
        &ops::editor::Submenu(SubmenuOp::Push),
    ),
    Keybind::nomod(SubmenuOp::Push, Char('p'), &ops::push::Push),
    // Rebase
    Keybind::nomod(
        SubmenuOp::None,
        Char('r'),
        &ops::editor::Submenu(SubmenuOp::Rebase),
    ),
    Keybind::nomod(
        SubmenuOp::Rebase,
        Char('i'),
        &ops::rebase::RebaseInteractive,
    ),
    Keybind::nomod(SubmenuOp::Rebase, Char('a'), &ops::rebase::RebaseAbort),
    Keybind::nomod(SubmenuOp::Rebase, Char('c'), &ops::rebase::RebaseContinue),
    Keybind::nomod(SubmenuOp::Rebase, Char('f'), &ops::rebase::RebaseAutosquash),
    // Reset
    Keybind::shift(
        SubmenuOp::None,
        Char('X'),
        &ops::editor::Submenu(SubmenuOp::Reset),
    ),
    Keybind::nomod(SubmenuOp::Reset, Char('s'), &ops::reset::ResetSoft),
    Keybind::nomod(SubmenuOp::Reset, Char('m'), &ops::reset::ResetMixed),
    Keybind::nomod(SubmenuOp::Reset, Char('h'), &ops::reset::ResetHard),
    // Show
    Keybind::nomod(SubmenuOp::None, Enter, &ops::show::Show),
    // Show refs
    Keybind::nomod(SubmenuOp::None, Char('y'), &ops::show_refs::ShowRefs),
    // Discard
    Keybind::shift(SubmenuOp::None, Char('K'), &ops::discard::Discard),
    // Target actions
    Keybind::nomod(SubmenuOp::None, Char('s'), &ops::stage::Stage),
    Keybind::nomod(SubmenuOp::None, Char('u'), &ops::unstage::Unstage),
];

pub(crate) fn op_of_key_event(
    pending: SubmenuOp,
    key: event::KeyEvent,
) -> Option<&'static dyn OpTrait> {
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
