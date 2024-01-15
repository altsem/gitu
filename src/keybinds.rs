use crossterm::event::{self, KeyCode, KeyModifiers};

type Mods = KeyModifiers;

pub(crate) const KEYBINDS: &[(KeyModifiers, KeyCode, Op)] = &[
    // Generic
    (Mods::NONE, KeyCode::Char('q'), Op::Quit),
    (Mods::NONE, KeyCode::Char('g'), Op::Refresh),
    // Navigation
    (Mods::NONE, KeyCode::Tab, Op::ToggleSection),
    (Mods::NONE, KeyCode::Char('k'), Op::SelectPrevious),
    (Mods::NONE, KeyCode::Char('j'), Op::SelectNext),
    (Mods::CONTROL, KeyCode::Char('u'), Op::HalfPageUp),
    (Mods::CONTROL, KeyCode::Char('d'), Op::HalfPageDown),
    // Commands
    (Mods::NONE, KeyCode::Char('l'), Op::Log),
    (Mods::NONE, KeyCode::Char('f'), Op::Fetch),
    (Mods::NONE, KeyCode::Char('c'), Op::Commit),
    (Mods::SHIFT, KeyCode::Char('P'), Op::Push),
    (Mods::NONE, KeyCode::Char('p'), Op::Pull),
    // Target actions
    (Mods::NONE, KeyCode::Enter, Op::Target(TargetOp::ShowOrEdit)),
    (Mods::NONE, KeyCode::Char('s'), Op::Target(TargetOp::Stage)),
    (
        Mods::NONE,
        KeyCode::Char('u'),
        Op::Target(TargetOp::Unstage),
    ),
];

#[derive(Clone, Copy)]
pub(crate) enum Op {
    Quit,
    Refresh,
    ToggleSection,
    SelectPrevious,
    SelectNext,
    HalfPageUp,
    HalfPageDown,
    Log,
    Fetch,
    Commit,
    Push,
    Pull,
    Target(TargetOp),
}

#[derive(Clone, Copy)]
pub(crate) enum TargetOp {
    ShowOrEdit,
    Stage,
    Unstage,
}

pub(crate) fn action_of_key_event(key: event::KeyEvent) -> Option<Op> {
    KEYBINDS
        .iter()
        .find(|&(modifiers, code, _)| (modifiers, code) == (&key.modifiers, &key.code))
        .map(|(_, _, action)| *action)
}
