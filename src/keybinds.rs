use crate::ops::Menu;
use crate::ops::Op;
use crossterm::event::{self, KeyCode, KeyModifiers};
use KeyCode::*;

pub(crate) struct Keybind {
    pub menu: Menu,
    pub mods: KeyModifiers,
    pub key: KeyCode,
    pub op: Op,
}

impl Keybind {
    const fn nomod(menu: Menu, key: KeyCode, op: Op) -> Self {
        Self {
            menu,
            mods: KeyModifiers::NONE,
            key,
            op,
        }
    }

    const fn ctrl(menu: Menu, key: KeyCode, op: Op) -> Self {
        Self {
            menu,
            mods: KeyModifiers::CONTROL,
            key,
            op,
        }
    }

    const fn shift(menu: Menu, key: KeyCode, op: Op) -> Self {
        Self {
            menu,
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
    Keybind::nomod(Menu::None, Char('q'), Op::Quit),
    Keybind::nomod(Menu::None, Esc, Op::Quit),
    Keybind::nomod(Menu::None, Char('g'), Op::Refresh),
    // Editor
    Keybind::nomod(Menu::None, Tab, Op::ToggleSection),
    Keybind::nomod(Menu::None, Char('k'), Op::MoveUp),
    Keybind::nomod(Menu::None, Char('p'), Op::MoveUp),
    Keybind::nomod(Menu::None, KeyCode::Up, Op::MoveUp),
    Keybind::nomod(Menu::None, Char('j'), Op::MoveDown),
    Keybind::nomod(Menu::None, Char('n'), Op::MoveDown),
    Keybind::nomod(Menu::None, KeyCode::Down, Op::MoveDown),
    Keybind::ctrl(Menu::None, Char('k'), Op::MoveUpLine),
    Keybind::ctrl(Menu::None, Char('p'), Op::MoveUpLine),
    Keybind::ctrl(Menu::None, KeyCode::Up, Op::MoveUpLine),
    Keybind::ctrl(Menu::None, Char('j'), Op::MoveDownLine),
    Keybind::ctrl(Menu::None, Char('n'), Op::MoveDownLine),
    Keybind::ctrl(Menu::None, KeyCode::Down, Op::MoveDownLine),
    Keybind::ctrl(Menu::None, Char('u'), Op::HalfPageUp),
    Keybind::ctrl(Menu::None, Char('d'), Op::HalfPageDown),
    // Help
    Keybind::nomod(Menu::None, Char('h'), Op::Menu(Menu::Help)),
    Keybind::nomod(Menu::Help, Char('q'), Op::Quit),
    Keybind::nomod(Menu::Help, Esc, Op::Quit),
    // Branch
    Keybind::nomod(Menu::None, Char('b'), Op::Menu(Menu::Branch)),
    Keybind::nomod(Menu::Branch, Char('b'), Op::Checkout),
    Keybind::nomod(Menu::Branch, Char('c'), Op::CheckoutNewBranch),
    Keybind::nomod(Menu::Branch, Char('q'), Op::Quit),
    Keybind::nomod(Menu::Branch, Esc, Op::Quit),
    // Commit
    Keybind::nomod(Menu::None, Char('c'), Op::Menu(Menu::Commit)),
    Keybind::nomod(Menu::Commit, Char('c'), Op::Commit),
    Keybind::nomod(Menu::Commit, Char('a'), Op::CommitAmend),
    Keybind::nomod(Menu::Commit, Char('f'), Op::CommitFixup),
    Keybind::nomod(Menu::Commit, Char('q'), Op::Quit),
    Keybind::nomod(Menu::Commit, Esc, Op::Quit),
    // Fetch
    Keybind::nomod(Menu::None, Char('f'), Op::Menu(Menu::Fetch)),
    Keybind::nomod(Menu::Fetch, Char('a'), Op::FetchAll),
    Keybind::nomod(Menu::Fetch, Char('q'), Op::Quit),
    Keybind::nomod(Menu::Fetch, Esc, Op::Quit),
    // Log
    Keybind::nomod(Menu::None, Char('l'), Op::Menu(Menu::Log)),
    Keybind::nomod(Menu::Log, Char('l'), Op::LogCurrent),
    Keybind::nomod(Menu::Log, Char('o'), Op::LogOther),
    Keybind::nomod(Menu::Log, Char('q'), Op::Quit),
    Keybind::nomod(Menu::Log, Esc, Op::Quit),
    // Pull
    Keybind::shift(Menu::None, Char('F'), Op::Menu(Menu::Pull)),
    Keybind::nomod(Menu::Pull, Char('p'), Op::Pull),
    Keybind::nomod(Menu::Pull, Char('q'), Op::Quit),
    Keybind::nomod(Menu::Pull, Esc, Op::Quit),
    // Push
    Keybind::shift(Menu::None, Char('P'), Op::Menu(Menu::Push)),
    Keybind::nomod(Menu::Push, Char('p'), Op::Push),
    Keybind::nomod(Menu::Push, Char('q'), Op::Quit),
    Keybind::nomod(Menu::Push, Esc, Op::Quit),
    // Rebase
    Keybind::nomod(Menu::None, Char('r'), Op::Menu(Menu::Rebase)),
    Keybind::nomod(Menu::Rebase, Char('i'), Op::RebaseInteractive),
    Keybind::nomod(Menu::Rebase, Char('a'), Op::RebaseAbort),
    Keybind::nomod(Menu::Rebase, Char('c'), Op::RebaseContinue),
    Keybind::nomod(Menu::Rebase, Char('f'), Op::RebaseAutosquash),
    Keybind::nomod(Menu::Rebase, Char('q'), Op::Quit),
    Keybind::nomod(Menu::Rebase, Esc, Op::Quit),
    // Reset
    Keybind::shift(Menu::None, Char('X'), Op::Menu(Menu::Reset)),
    Keybind::nomod(Menu::Reset, Char('s'), Op::ResetSoft),
    Keybind::nomod(Menu::Reset, Char('m'), Op::ResetMixed),
    Keybind::nomod(Menu::Reset, Char('h'), Op::ResetHard),
    Keybind::nomod(Menu::Reset, Char('q'), Op::Quit),
    Keybind::nomod(Menu::Reset, Esc, Op::Quit),
    // Show
    Keybind::nomod(Menu::None, Enter, Op::Show),
    // Show refs
    Keybind::nomod(Menu::None, Char('y'), Op::ShowRefs),
    // Stash
    Keybind::nomod(Menu::None, Char('z'), Op::Menu(Menu::Stash)),
    Keybind::nomod(Menu::Stash, Char('z'), Op::Stash),
    Keybind::nomod(Menu::Stash, Char('i'), Op::StashIndex),
    Keybind::nomod(Menu::Stash, Char('w'), Op::StashWorktree),
    Keybind::nomod(Menu::Stash, Char('x'), Op::StashKeepIndex),
    Keybind::nomod(Menu::Stash, Char('p'), Op::StashPop),
    Keybind::nomod(Menu::Stash, Char('a'), Op::StashApply),
    Keybind::nomod(Menu::Stash, Char('k'), Op::StashDrop),
    Keybind::nomod(Menu::Stash, Char('q'), Op::Quit),
    Keybind::nomod(Menu::Stash, Esc, Op::Quit),
    // Discard
    Keybind::shift(Menu::None, Char('K'), Op::Discard),
    // Target actions
    Keybind::nomod(Menu::None, Char('s'), Op::Stage),
    Keybind::nomod(Menu::None, Char('u'), Op::Unstage),
];

pub(crate) fn op_of_key_event(pending: Menu, key: event::KeyEvent) -> Option<Op> {
    KEYBINDS
        .iter()
        .find(|keybind| {
            (keybind.menu, keybind.mods, keybind.key) == (pending, key.modifiers, key.code)
        })
        .map(|keybind| keybind.op)
}

pub(crate) fn list(pending: &Menu) -> impl Iterator<Item = &Keybind> {
    let expected = if pending == &Menu::Help {
        Menu::None
    } else {
        *pending
    };

    KEYBINDS
        .iter()
        .filter(move |keybind| keybind.menu == expected)
}
