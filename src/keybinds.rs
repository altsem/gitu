use crate::{menu::Menu, ops::Op};
use crossterm::event::{self, KeyCode, KeyModifiers};
use KeyCode::*;

pub(crate) struct Keybind {
    pub menu: Option<Menu>,
    pub mods: KeyModifiers,
    pub key: KeyCode,
    pub op: Op,
}

impl Keybind {
    const fn nomod(menu: Option<Menu>, key: KeyCode, op: Op) -> Self {
        Self {
            menu,
            mods: KeyModifiers::NONE,
            key,
            op,
        }
    }

    const fn ctrl(menu: Option<Menu>, key: KeyCode, op: Op) -> Self {
        Self {
            menu,
            mods: KeyModifiers::CONTROL,
            key,
            op,
        }
    }

    const fn shift(menu: Option<Menu>, key: KeyCode, op: Op) -> Self {
        Self {
            menu,
            mods: KeyModifiers::SHIFT,
            key,
            op,
        }
    }

    pub(crate) fn format_key(&self) -> String {
        let prefix = if matches!(self.op, Op::ToggleArg(_)) {
            "-".to_string()
        } else {
            self.mods
                .iter_names()
                .map(|(name, _)| match name {
                    "CONTROL" => "C-",
                    "SHIFT" => "",
                    _ => unimplemented!("format_key mod {}", name),
                })
                .collect::<String>()
        };

        prefix
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
    Keybind::nomod(None, Char('q'), Op::Quit),
    Keybind::nomod(None, Esc, Op::Quit),
    Keybind::nomod(None, Char('g'), Op::Refresh),
    // Editor
    Keybind::nomod(None, Tab, Op::ToggleSection),
    Keybind::nomod(None, Char('k'), Op::MoveUp),
    Keybind::nomod(None, Char('p'), Op::MoveUp),
    Keybind::nomod(None, KeyCode::Up, Op::MoveUp),
    Keybind::nomod(None, Char('j'), Op::MoveDown),
    Keybind::nomod(None, Char('n'), Op::MoveDown),
    Keybind::nomod(None, KeyCode::Down, Op::MoveDown),
    Keybind::ctrl(None, Char('k'), Op::MoveUpLine),
    Keybind::ctrl(None, Char('p'), Op::MoveUpLine),
    Keybind::ctrl(None, KeyCode::Up, Op::MoveUpLine),
    Keybind::ctrl(None, Char('j'), Op::MoveDownLine),
    Keybind::ctrl(None, Char('n'), Op::MoveDownLine),
    Keybind::ctrl(None, KeyCode::Down, Op::MoveDownLine),
    Keybind::ctrl(None, Char('u'), Op::HalfPageUp),
    Keybind::ctrl(None, Char('d'), Op::HalfPageDown),
    // Help
    Keybind::nomod(None, Char('h'), Op::Menu(Menu::Help)),
    Keybind::nomod(Some(Menu::Help), Char('q'), Op::Quit),
    Keybind::nomod(Some(Menu::Help), Esc, Op::Quit),
    // Branch
    Keybind::nomod(None, Char('b'), Op::Menu(Menu::Branch)),
    Keybind::nomod(Some(Menu::Branch), Char('b'), Op::Checkout),
    Keybind::nomod(Some(Menu::Branch), Char('c'), Op::CheckoutNewBranch),
    Keybind::nomod(Some(Menu::Branch), Char('q'), Op::Quit),
    Keybind::nomod(Some(Menu::Branch), Esc, Op::Quit),
    // Commit
    Keybind::nomod(None, Char('c'), Op::Menu(Menu::Commit)),
    Keybind::nomod(Some(Menu::Commit), Char('c'), Op::Commit),
    Keybind::nomod(Some(Menu::Commit), Char('a'), Op::CommitAmend),
    Keybind::nomod(Some(Menu::Commit), Char('f'), Op::CommitFixup),
    Keybind::nomod(Some(Menu::Commit), Char('q'), Op::Quit),
    Keybind::nomod(Some(Menu::Commit), Esc, Op::Quit),
    // Fetch
    Keybind::nomod(None, Char('f'), Op::Menu(Menu::Fetch)),
    Keybind::nomod(Some(Menu::Fetch), Char('a'), Op::FetchAll),
    Keybind::nomod(Some(Menu::Fetch), Char('q'), Op::Quit),
    Keybind::nomod(Some(Menu::Fetch), Esc, Op::Quit),
    // Log
    Keybind::nomod(None, Char('l'), Op::Menu(Menu::Log)),
    Keybind::nomod(Some(Menu::Log), Char('l'), Op::LogCurrent),
    Keybind::nomod(Some(Menu::Log), Char('o'), Op::LogOther),
    Keybind::nomod(Some(Menu::Log), Char('q'), Op::Quit),
    Keybind::nomod(Some(Menu::Log), Esc, Op::Quit),
    // Pull
    Keybind::shift(None, Char('F'), Op::Menu(Menu::Pull)),
    Keybind::nomod(Some(Menu::Pull), Char('p'), Op::Pull),
    Keybind::nomod(Some(Menu::Pull), Char('q'), Op::Quit),
    Keybind::nomod(Some(Menu::Pull), Esc, Op::Quit),
    // Push
    Keybind::shift(None, Char('P'), Op::Menu(Menu::Push)),
    Keybind::nomod(
        Some(Menu::Push),
        Char('f'),
        Op::ToggleArg("--force-with-lease"),
    ),
    Keybind::nomod(Some(Menu::Push), Char('p'), Op::Push),
    Keybind::nomod(Some(Menu::Push), Char('q'), Op::Quit),
    Keybind::nomod(Some(Menu::Push), Esc, Op::Quit),
    // Rebase
    Keybind::nomod(None, Char('r'), Op::Menu(Menu::Rebase)),
    Keybind::nomod(Some(Menu::Rebase), Char('i'), Op::RebaseInteractive),
    Keybind::nomod(Some(Menu::Rebase), Char('a'), Op::RebaseAbort),
    Keybind::nomod(Some(Menu::Rebase), Char('c'), Op::RebaseContinue),
    Keybind::nomod(Some(Menu::Rebase), Char('f'), Op::RebaseAutosquash),
    Keybind::nomod(Some(Menu::Rebase), Char('q'), Op::Quit),
    Keybind::nomod(Some(Menu::Rebase), Esc, Op::Quit),
    // Reset
    Keybind::shift(None, Char('X'), Op::Menu(Menu::Reset)),
    Keybind::nomod(Some(Menu::Reset), Char('s'), Op::ResetSoft),
    Keybind::nomod(Some(Menu::Reset), Char('m'), Op::ResetMixed),
    Keybind::nomod(Some(Menu::Reset), Char('h'), Op::ResetHard),
    Keybind::nomod(Some(Menu::Reset), Char('q'), Op::Quit),
    Keybind::nomod(Some(Menu::Reset), Esc, Op::Quit),
    // Show
    Keybind::nomod(None, Enter, Op::Show),
    // Show refs
    Keybind::nomod(None, Char('y'), Op::ShowRefs),
    // Stash
    Keybind::nomod(None, Char('z'), Op::Menu(Menu::Stash)),
    Keybind::nomod(Some(Menu::Stash), Char('z'), Op::Stash),
    Keybind::nomod(Some(Menu::Stash), Char('i'), Op::StashIndex),
    Keybind::nomod(Some(Menu::Stash), Char('w'), Op::StashWorktree),
    Keybind::nomod(Some(Menu::Stash), Char('x'), Op::StashKeepIndex),
    Keybind::nomod(Some(Menu::Stash), Char('p'), Op::StashPop),
    Keybind::nomod(Some(Menu::Stash), Char('a'), Op::StashApply),
    Keybind::nomod(Some(Menu::Stash), Char('k'), Op::StashDrop),
    Keybind::nomod(Some(Menu::Stash), Char('q'), Op::Quit),
    Keybind::nomod(Some(Menu::Stash), Esc, Op::Quit),
    // Discard
    Keybind::shift(None, Char('K'), Op::Discard),
    // Target actions
    Keybind::nomod(None, Char('s'), Op::Stage),
    Keybind::nomod(None, Char('u'), Op::Unstage),
];

pub(crate) fn op_of_key_event(pending: Option<Menu>, key: event::KeyEvent) -> Option<Op> {
    KEYBINDS
        .iter()
        .filter(|keybind| !matches!(keybind.op, Op::ToggleArg(_)))
        .find(|keybind| {
            (keybind.menu, keybind.mods, keybind.key) == (pending, key.modifiers, key.code)
        })
        .map(|keybind| keybind.op)
}

pub(crate) fn arg_op_of_key_event(pending: Option<Menu>, key: event::KeyEvent) -> Option<Op> {
    KEYBINDS
        .iter()
        .filter(|keybind| matches!(keybind.op, Op::ToggleArg(_)))
        .find(|keybind| {
            (keybind.menu, keybind.mods, keybind.key) == (pending, key.modifiers, key.code)
        })
        .map(|keybind| keybind.op)
}

pub(crate) fn list(pending: &Menu) -> impl Iterator<Item = &Keybind> {
    let expected = if pending == &Menu::Help {
        None
    } else {
        Some(*pending)
    };

    KEYBINDS
        .iter()
        .filter(|keybind| !matches!(keybind.op, Op::ToggleArg(_)))
        .filter(move |keybind| keybind.menu == expected)
}

pub(crate) fn arg_list(pending: &Menu) -> impl Iterator<Item = &Keybind> {
    let expected = if pending == &Menu::Help {
        None
    } else {
        Some(*pending)
    };

    KEYBINDS
        .iter()
        .filter(|keybind| matches!(keybind.op, Op::ToggleArg(_)))
        .filter(move |keybind| keybind.menu == expected)
}
