use crate::ops::Op;
use crate::ops::SubmenuOp;
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
    Keybind::nomod(SubmenuOp::None, Char('q'), Op::Quit),
    Keybind::nomod(SubmenuOp::None, Esc, Op::Quit),
    Keybind::nomod(SubmenuOp::None, Char('g'), Op::Refresh),
    // Editor
    Keybind::nomod(SubmenuOp::None, Tab, Op::ToggleSection),
    Keybind::nomod(SubmenuOp::None, Char('k'), Op::MoveUp),
    Keybind::nomod(SubmenuOp::None, Char('p'), Op::MoveUp),
    Keybind::nomod(SubmenuOp::None, KeyCode::Up, Op::MoveUp),
    Keybind::nomod(SubmenuOp::None, Char('j'), Op::MoveDown),
    Keybind::nomod(SubmenuOp::None, Char('n'), Op::MoveDown),
    Keybind::nomod(SubmenuOp::None, KeyCode::Down, Op::MoveDown),
    Keybind::ctrl(SubmenuOp::None, Char('k'), Op::MoveUpLine),
    Keybind::ctrl(SubmenuOp::None, Char('p'), Op::MoveUpLine),
    Keybind::ctrl(SubmenuOp::None, KeyCode::Up, Op::MoveUpLine),
    Keybind::ctrl(SubmenuOp::None, Char('j'), Op::MoveDownLine),
    Keybind::ctrl(SubmenuOp::None, Char('n'), Op::MoveDownLine),
    Keybind::ctrl(SubmenuOp::None, KeyCode::Down, Op::MoveDownLine),
    Keybind::ctrl(SubmenuOp::None, Char('u'), Op::HalfPageUp),
    Keybind::ctrl(SubmenuOp::None, Char('d'), Op::HalfPageDown),
    // Help
    Keybind::nomod(SubmenuOp::None, Char('h'), Op::Submenu(SubmenuOp::Help)),
    Keybind::nomod(SubmenuOp::Help, Char('q'), Op::Quit),
    Keybind::nomod(SubmenuOp::Help, Esc, Op::Quit),
    // Branch
    Keybind::nomod(SubmenuOp::None, Char('b'), Op::Submenu(SubmenuOp::Branch)),
    Keybind::nomod(SubmenuOp::Branch, Char('b'), Op::Checkout),
    Keybind::nomod(SubmenuOp::Branch, Char('c'), Op::CheckoutNewBranch),
    Keybind::nomod(SubmenuOp::Branch, Char('q'), Op::Quit),
    Keybind::nomod(SubmenuOp::Branch, Esc, Op::Quit),
    // Commit
    Keybind::nomod(SubmenuOp::None, Char('c'), Op::Submenu(SubmenuOp::Commit)),
    Keybind::nomod(SubmenuOp::Commit, Char('c'), Op::Commit),
    Keybind::nomod(SubmenuOp::Commit, Char('a'), Op::CommitAmend),
    Keybind::nomod(SubmenuOp::Commit, Char('f'), Op::CommitFixup),
    Keybind::nomod(SubmenuOp::Commit, Char('q'), Op::Quit),
    Keybind::nomod(SubmenuOp::Commit, Esc, Op::Quit),
    // Fetch
    Keybind::nomod(SubmenuOp::None, Char('f'), Op::Submenu(SubmenuOp::Fetch)),
    Keybind::nomod(SubmenuOp::Fetch, Char('a'), Op::FetchAll),
    Keybind::nomod(SubmenuOp::Fetch, Char('q'), Op::Quit),
    Keybind::nomod(SubmenuOp::Fetch, Esc, Op::Quit),
    // Log
    Keybind::nomod(SubmenuOp::None, Char('l'), Op::Submenu(SubmenuOp::Log)),
    Keybind::nomod(SubmenuOp::Log, Char('l'), Op::LogCurrent),
    Keybind::nomod(SubmenuOp::Log, Char('o'), Op::LogOther),
    Keybind::nomod(SubmenuOp::Log, Char('q'), Op::Quit),
    Keybind::nomod(SubmenuOp::Log, Esc, Op::Quit),
    // Pull
    Keybind::shift(SubmenuOp::None, Char('F'), Op::Submenu(SubmenuOp::Pull)),
    Keybind::nomod(SubmenuOp::Pull, Char('p'), Op::Pull),
    Keybind::nomod(SubmenuOp::Pull, Char('q'), Op::Quit),
    Keybind::nomod(SubmenuOp::Pull, Esc, Op::Quit),
    // Push
    Keybind::shift(SubmenuOp::None, Char('P'), Op::Submenu(SubmenuOp::Push)),
    Keybind::nomod(SubmenuOp::Push, Char('p'), Op::Push),
    Keybind::nomod(SubmenuOp::Push, Char('q'), Op::Quit),
    Keybind::nomod(SubmenuOp::Push, Esc, Op::Quit),
    // Rebase
    Keybind::nomod(SubmenuOp::None, Char('r'), Op::Submenu(SubmenuOp::Rebase)),
    Keybind::nomod(SubmenuOp::Rebase, Char('i'), Op::RebaseInteractive),
    Keybind::nomod(SubmenuOp::Rebase, Char('a'), Op::RebaseAbort),
    Keybind::nomod(SubmenuOp::Rebase, Char('c'), Op::RebaseContinue),
    Keybind::nomod(SubmenuOp::Rebase, Char('f'), Op::RebaseAutosquash),
    Keybind::nomod(SubmenuOp::Rebase, Char('q'), Op::Quit),
    Keybind::nomod(SubmenuOp::Rebase, Esc, Op::Quit),
    // Reset
    Keybind::shift(SubmenuOp::None, Char('X'), Op::Submenu(SubmenuOp::Reset)),
    Keybind::nomod(SubmenuOp::Reset, Char('s'), Op::ResetSoft),
    Keybind::nomod(SubmenuOp::Reset, Char('m'), Op::ResetMixed),
    Keybind::nomod(SubmenuOp::Reset, Char('h'), Op::ResetHard),
    Keybind::nomod(SubmenuOp::Reset, Char('q'), Op::Quit),
    Keybind::nomod(SubmenuOp::Reset, Esc, Op::Quit),
    // Show
    Keybind::nomod(SubmenuOp::None, Enter, Op::Show),
    // Show refs
    Keybind::nomod(SubmenuOp::None, Char('y'), Op::ShowRefs),
    // Stash
    Keybind::nomod(SubmenuOp::None, Char('z'), Op::Submenu(SubmenuOp::Stash)),
    Keybind::nomod(SubmenuOp::Stash, Char('z'), Op::Stash),
    Keybind::nomod(SubmenuOp::Stash, Char('i'), Op::StashIndex),
    Keybind::nomod(SubmenuOp::Stash, Char('w'), Op::StashWorktree),
    Keybind::nomod(SubmenuOp::Stash, Char('x'), Op::StashKeepIndex),
    Keybind::nomod(SubmenuOp::Stash, Char('p'), Op::StashPop),
    Keybind::nomod(SubmenuOp::Stash, Char('a'), Op::StashApply),
    Keybind::nomod(SubmenuOp::Stash, Char('k'), Op::StashDrop),
    Keybind::nomod(SubmenuOp::Stash, Char('q'), Op::Quit),
    Keybind::nomod(SubmenuOp::Stash, Esc, Op::Quit),
    // Discard
    Keybind::shift(SubmenuOp::None, Char('K'), Op::Discard),
    // Target actions
    Keybind::nomod(SubmenuOp::None, Char('s'), Op::Stage),
    Keybind::nomod(SubmenuOp::None, Char('u'), Op::Unstage),
];

pub(crate) fn op_of_key_event(pending: SubmenuOp, key: event::KeyEvent) -> Option<Op> {
    KEYBINDS
        .iter()
        .find(|keybind| {
            (keybind.submenu, keybind.mods, keybind.key) == (pending, key.modifiers, key.code)
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
