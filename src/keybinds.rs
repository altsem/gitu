use crossterm::event::{self, KeyCode, KeyModifiers};

type Mods = KeyModifiers;

use KeyCode::*;
use Op::*;
use TargetOp::*;
use TransientOp::*;

pub(crate) struct Keybind {
    pub transient: Option<TransientOp>,
    pub mods: KeyModifiers,
    pub key: KeyCode,
    pub op: Op,
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
    Keybind::new(None, Mods::NONE, Char('q'), Quit),
    Keybind::new(None, Mods::NONE, Char('g'), Refresh),
    Keybind::new(None, Mods::NONE, Tab, ToggleSection),
    // Navigation
    Keybind::new(None, Mods::NONE, Char('k'), SelectPrevious),
    Keybind::new(None, Mods::NONE, Char('j'), SelectNext),
    Keybind::new(None, Mods::CONTROL, Char('u'), HalfPageUp),
    Keybind::new(None, Mods::CONTROL, Char('d'), HalfPageDown),
    // Help
    Keybind::new(None, Mods::NONE, Char('h'), Transient(Help)),
    Keybind::new(Some(Help), Mods::NONE, Char('q'), Quit),
    // Commit
    Keybind::new(None, Mods::NONE, Char('c'), Transient(TransientOp::Commit)),
    Keybind::new(Some(TransientOp::Commit), Mods::NONE, Char('c'), Op::Commit),
    Keybind::new(Some(TransientOp::Commit), Mods::NONE, Char('q'), Quit),
    // Fetch
    Keybind::new(None, Mods::NONE, Char('f'), Transient(Fetch)),
    Keybind::new(Some(Fetch), Mods::NONE, Char('a'), FetchAll),
    Keybind::new(Some(Fetch), Mods::NONE, Char('q'), Quit),
    // Log
    Keybind::new(None, Mods::NONE, Char('l'), Transient(Log)),
    Keybind::new(Some(Log), Mods::NONE, Char('l'), LogCurrent),
    Keybind::new(Some(Log), Mods::NONE, Char('q'), Quit),
    // Pull
    Keybind::new(None, Mods::SHIFT, Char('F'), Transient(Pull)),
    Keybind::new(Some(Pull), Mods::NONE, Char('p'), PullRemote),
    Keybind::new(Some(Pull), Mods::NONE, Char('q'), Quit),
    // Push
    Keybind::new(None, Mods::SHIFT, Char('P'), Transient(Push)),
    Keybind::new(Some(Push), Mods::NONE, Char('p'), PushRemote),
    Keybind::new(Some(Push), Mods::NONE, Char('q'), Quit),
    // Rebase
    Keybind::new(None, Mods::NONE, Char('r'), Transient(TransientOp::Rebase)),
    Keybind::new(
        Some(TransientOp::Rebase),
        Mods::NONE,
        Char('i'),
        Target(TargetOp::RebaseInteractive),
    ),
    Keybind::new(Some(TransientOp::Rebase), Mods::NONE, Char('q'), Quit),
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
    PushRemote,
    PullRemote,
    Transient(TransientOp),
    Commit,
    FetchAll,
    LogCurrent,
    Target(TargetOp),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum TransientOp {
    Commit,
    Fetch,
    Help,
    Log,
    Pull,
    Push,
    Rebase,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum TargetOp {
    Show,
    Stage,
    Unstage,
    CopyToClipboard,
    RebaseInteractive,
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
    let expected = if op == &Help { None } else { Some(*op) };

    KEYBINDS
        .iter()
        .filter(move |keybind| keybind.transient == expected)
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
