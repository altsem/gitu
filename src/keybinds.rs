use crossterm::event::{self, KeyCode, KeyModifiers};
use std::fmt::Display;

type Mods = KeyModifiers;

use KeyCode::*;
use Op::*;
use TargetOp::*;
use TransientOp::*;

pub(crate) struct Keybind {
    transient: Option<TransientOp>,
    mods: KeyModifiers,
    key: KeyCode,
    op: Op,
}

impl Keybind {
    const fn new(transient: Option<TransientOp>, mods: KeyModifiers, key: KeyCode, op: Op) -> Self {
        Self {
            transient,
            mods,
            key,
            op,
        }
    }
}

impl Display for Keybind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "{} {:?}",
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
            },
            self.op
        ))
    }
}

pub(crate) const KEYBINDS: &[Keybind] = &[
    // Generic
    Keybind::new(None, Mods::NONE, Char('q'), Quit),
    Keybind::new(None, Mods::NONE, Char('g'), Refresh),
    Keybind::new(None, Mods::NONE, Tab, ToggleSection),
    // Navigation
    Keybind::new(None, Mods::NONE, Char('k'), SelectPrevious),
    Keybind::new(None, Mods::NONE, Char('j'), SelectNext),
    Keybind::new(None, Mods::CONTROL, Char('u'), HalfPageUp),
    Keybind::new(None, Mods::CONTROL, Char('d'), HalfPageDown),
    // TransientOps
    Keybind::new(None, Mods::NONE, Char('l'), Transient(Log)),
    Keybind::new(Some(Log), Mods::NONE, Char('q'), Quit),
    Keybind::new(Some(Log), Mods::NONE, Char('l'), LogCurrent),
    // TODO Make these transient
    Keybind::new(None, Mods::NONE, Char('f'), Fetch),
    Keybind::new(None, Mods::NONE, Char('c'), Commit),
    Keybind::new(None, Mods::SHIFT, Char('P'), Push),
    Keybind::new(None, Mods::NONE, Char('p'), Pull),
    // Target actions
    Keybind::new(None, Mods::NONE, Enter, Target(Show)),
    Keybind::new(None, Mods::NONE, Char('s'), Target(Stage)),
    Keybind::new(None, Mods::NONE, Char('u'), Target(Unstage)),
    Keybind::new(None, Mods::NONE, Char('y'), Target(CopyToClipboard)),
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
    Fetch,
    Commit,
    Push,
    Pull,
    Transient(TransientOp),
    LogCurrent,
    Target(TargetOp),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum TransientOp {
    Log,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum TargetOp {
    Show,
    Stage,
    Unstage,
    CopyToClipboard,
}

impl TargetOp {
    pub(crate) fn list_all() -> impl Iterator<Item = TargetOp> {
        [TargetOp::Show, TargetOp::Stage, TargetOp::Unstage].into_iter()
    }
}

pub(crate) fn op_of_key_event(pending: Option<TransientOp>, key: event::KeyEvent) -> Option<Op> {
    KEYBINDS
        .iter()
        .find(|keybind| {
            (keybind.transient, keybind.mods, keybind.key) == (pending, key.modifiers, key.code)
        })
        .map(|keybind| keybind.op)
}

pub(crate) fn list_transient_binds(op: &TransientOp) -> impl Iterator<Item = &Keybind> {
    KEYBINDS
        .iter()
        .filter(|keybind| keybind.transient == Some(*op))
}

pub(crate) fn display_key(pending: Option<TransientOp>, op: Op) -> Option<String> {
    KEYBINDS
        .iter()
        .find(|keybind| keybind.transient == pending && keybind.op == op)
        .map(|keybind| match keybind.key {
            KeyCode::Enter => "ret".to_string(),
            KeyCode::Left => "←".to_string(),
            KeyCode::Right => "→".to_string(),
            KeyCode::Up => "↑".to_string(),
            KeyCode::Down => "↓".to_string(),
            KeyCode::Tab => "tab".to_string(),
            KeyCode::Delete => "del".to_string(),
            KeyCode::Insert => "ins".to_string(),
            KeyCode::F(n) => format!("F{}", n),
            KeyCode::Char(c) => if keybind.mods.contains(KeyModifiers::SHIFT) {
                c.to_ascii_uppercase()
            } else {
                c
            }
            .to_string(),
            KeyCode::Esc => "esc".to_string(),
            _ => "???".to_string(),
        })
}
