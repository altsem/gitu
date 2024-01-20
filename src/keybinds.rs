use crossterm::event::{self, KeyCode, KeyModifiers};

type Mods = KeyModifiers;

use KeyCode::*;
use Op::*;
use TargetOp::*;

pub(crate) const KEYBINDS: &[(KeyModifiers, KeyCode, Op)] = &[
    // Generic
    (Mods::NONE, Char('q'), Quit),
    (Mods::NONE, Char('g'), Refresh),
    (Mods::NONE, Tab, ToggleSection),
    // Navigation
    (Mods::NONE, Char('k'), SelectPrevious),
    (Mods::NONE, Char('j'), SelectNext),
    (Mods::CONTROL, Char('u'), HalfPageUp),
    (Mods::CONTROL, Char('d'), HalfPageDown),
    // Commands
    (Mods::NONE, Char('l'), Log),
    (Mods::NONE, Char('f'), Fetch),
    (Mods::NONE, Char('c'), Commit),
    (Mods::SHIFT, Char('P'), Push),
    (Mods::NONE, Char('p'), Pull),
    // Target actions
    (Mods::NONE, Enter, Target(Show)),
    (Mods::NONE, Char('s'), Target(Stage)),
    (Mods::NONE, Char('u'), Target(Unstage)),
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
    Log,
    Fetch,
    Commit,
    Push,
    Pull,
    Target(TargetOp),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum TargetOp {
    Show,
    Stage,
    Unstage,
}

pub(crate) fn op_of_key_event(key: event::KeyEvent) -> Option<Op> {
    KEYBINDS
        .iter()
        .find(|&(modifiers, code, _)| (modifiers, code) == (&key.modifiers, &key.code))
        .map(|(_, _, action)| *action)
}

pub(crate) fn display_key(op: Op) -> Option<String> {
    KEYBINDS
        .iter()
        .find(|&(_, _, bound_op)| bound_op == &op)
        .map(|(modifiers, code, _)| match code {
            KeyCode::Enter => "RET".to_string(),
            KeyCode::Left => "←".to_string(),
            KeyCode::Right => "→".to_string(),
            KeyCode::Up => "↑".to_string(),
            KeyCode::Down => "↓".to_string(),
            KeyCode::Tab => "TAB".to_string(),
            KeyCode::Delete => "DEL".to_string(),
            KeyCode::Insert => "INS".to_string(),
            KeyCode::F(n) => format!("F{}", n),
            KeyCode::Char(c) => if modifiers.contains(KeyModifiers::SHIFT) {
                c.to_ascii_uppercase()
            } else {
                *c
            }
            .to_string(),
            KeyCode::Esc => "ESC".to_string(),
            _ => "???".to_string(),
        })
}
