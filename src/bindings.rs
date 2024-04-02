use crate::{key_parser, menu::Menu, ops::Op};
use crossterm::event::{KeyCode, KeyModifiers};

pub(crate) struct Binding {
    pub menu: Option<Menu>,
    pub raw: String,
    pub keys: Vec<(KeyModifiers, KeyCode)>,
    pub op: Op,
}

impl Binding {
    fn new(menu: Option<Menu>, raw_keys: &str, op: Op) -> Self {
        let ("", keys) = key_parser::parse_keys(raw_keys)
            .unwrap_or_else(|_| panic!("Couldn't parse keys: {}", raw_keys))
        else {
            unreachable!();
        };

        Self {
            menu,
            raw: raw_keys.to_string(),
            keys,
            op,
        }
    }
}

pub(crate) fn bindings() -> Vec<Binding> {
    vec![
        // Generic
        Binding::new(None, "q", Op::Quit),
        Binding::new(None, "<esc>", Op::Quit),
        Binding::new(None, "g", Op::Refresh),
        // Editor
        Binding::new(None, "<tab>", Op::ToggleSection),
        Binding::new(None, "k", Op::MoveUp),
        Binding::new(None, "p", Op::MoveUp),
        Binding::new(None, "<up>", Op::MoveUp),
        Binding::new(None, "j", Op::MoveDown),
        Binding::new(None, "n", Op::MoveDown),
        Binding::new(None, "<down>", Op::MoveDown),
        Binding::new(None, "<ctrl+k>", Op::MoveUpLine),
        Binding::new(None, "<ctrl+p>", Op::MoveUpLine),
        Binding::new(None, "<ctrl+up>", Op::MoveUpLine),
        Binding::new(None, "<ctrl+j>", Op::MoveDownLine),
        Binding::new(None, "<ctrl+n>", Op::MoveDownLine),
        Binding::new(None, "<ctrl+down>", Op::MoveDownLine),
        Binding::new(None, "<ctrl+u>", Op::HalfPageUp),
        Binding::new(None, "<ctrl+d>", Op::HalfPageDown),
        // Help
        Binding::new(None, "h", Op::Menu(Menu::Help)),
        Binding::new(Some(Menu::Help), "q", Op::Quit),
        Binding::new(Some(Menu::Help), "<esc>", Op::Quit),
        // Branch
        Binding::new(None, "b", Op::Menu(Menu::Branch)),
        Binding::new(Some(Menu::Branch), "b", Op::Checkout),
        Binding::new(Some(Menu::Branch), "c", Op::CheckoutNewBranch),
        Binding::new(Some(Menu::Branch), "q", Op::Quit),
        Binding::new(Some(Menu::Branch), "<esc>", Op::Quit),
        // Commit
        Binding::new(None, "c", Op::Menu(Menu::Commit)),
        Binding::new(Some(Menu::Commit), "c", Op::Commit),
        Binding::new(Some(Menu::Commit), "a", Op::CommitAmend),
        Binding::new(Some(Menu::Commit), "f", Op::CommitFixup),
        Binding::new(Some(Menu::Commit), "q", Op::Quit),
        Binding::new(Some(Menu::Commit), "<esc>", Op::Quit),
        // Fetch
        Binding::new(None, "f", Op::Menu(Menu::Fetch)),
        Binding::new(Some(Menu::Fetch), "a", Op::FetchAll),
        Binding::new(Some(Menu::Fetch), "q", Op::Quit),
        Binding::new(Some(Menu::Fetch), "<esc>", Op::Quit),
        // Log
        Binding::new(None, "l", Op::Menu(Menu::Log)),
        Binding::new(Some(Menu::Log), "l", Op::LogCurrent),
        Binding::new(Some(Menu::Log), "o", Op::LogOther),
        Binding::new(Some(Menu::Log), "q", Op::Quit),
        Binding::new(Some(Menu::Log), "<esc>", Op::Quit),
        // Pull
        Binding::new(None, "F", Op::Menu(Menu::Pull)),
        Binding::new(Some(Menu::Pull), "p", Op::Pull),
        Binding::new(Some(Menu::Pull), "q", Op::Quit),
        Binding::new(Some(Menu::Pull), "<esc>", Op::Quit),
        // Push
        Binding::new(None, "P", Op::Menu(Menu::Push)),
        Binding::new(Some(Menu::Push), "-f", Op::ToggleArg("--force-with-lease")),
        Binding::new(Some(Menu::Push), "p", Op::Push),
        Binding::new(Some(Menu::Push), "q", Op::Quit),
        Binding::new(Some(Menu::Push), "<esc>", Op::Quit),
        // Rebase
        Binding::new(None, "r", Op::Menu(Menu::Rebase)),
        Binding::new(Some(Menu::Rebase), "i", Op::RebaseInteractive),
        Binding::new(Some(Menu::Rebase), "a", Op::RebaseAbort),
        Binding::new(Some(Menu::Rebase), "c", Op::RebaseContinue),
        Binding::new(Some(Menu::Rebase), "e", Op::RebaseElsewhere),
        Binding::new(Some(Menu::Rebase), "f", Op::RebaseAutosquash),
        Binding::new(Some(Menu::Rebase), "q", Op::Quit),
        Binding::new(Some(Menu::Rebase), "<esc>", Op::Quit),
        // Reset
        Binding::new(None, "X", Op::Menu(Menu::Reset)),
        Binding::new(Some(Menu::Reset), "s", Op::ResetSoft),
        Binding::new(Some(Menu::Reset), "m", Op::ResetMixed),
        Binding::new(Some(Menu::Reset), "h", Op::ResetHard),
        Binding::new(Some(Menu::Reset), "q", Op::Quit),
        Binding::new(Some(Menu::Reset), "<esc>", Op::Quit),
        // Show
        Binding::new(None, "<enter>", Op::Show),
        // Show refs
        Binding::new(None, "y", Op::ShowRefs),
        // Stash
        Binding::new(None, "z", Op::Menu(Menu::Stash)),
        Binding::new(Some(Menu::Stash), "z", Op::Stash),
        Binding::new(Some(Menu::Stash), "i", Op::StashIndex),
        Binding::new(Some(Menu::Stash), "w", Op::StashWorktree),
        Binding::new(Some(Menu::Stash), "x", Op::StashKeepIndex),
        Binding::new(Some(Menu::Stash), "p", Op::StashPop),
        Binding::new(Some(Menu::Stash), "a", Op::StashApply),
        Binding::new(Some(Menu::Stash), "k", Op::StashDrop),
        Binding::new(Some(Menu::Stash), "q", Op::Quit),
        Binding::new(Some(Menu::Stash), "<esc>", Op::Quit),
        // Discard
        Binding::new(None, "K", Op::Discard),
        // Target actions
        Binding::new(None, "s", Op::Stage),
        Binding::new(None, "u", Op::Unstage),
    ]
}

pub(crate) fn match_bindings<'a>(
    bindings: &'a [Binding],
    pending: &'a Option<Menu>,
    events: &'a [(KeyModifiers, KeyCode)],
) -> impl Iterator<Item = &'a Binding> + 'a {
    bindings
        .iter()
        // TODO Support multiple keys in a sequence
        .filter(move |binding| &binding.menu == pending)
        .filter(|binding| binding.keys.starts_with(events))
}

pub(crate) fn list<'a>(
    bindings: &'a [Binding],
    pending: &Menu,
) -> impl Iterator<Item = &'a Binding> {
    let expected = if pending == &Menu::Help {
        None
    } else {
        Some(*pending)
    };

    bindings
        .iter()
        .filter(|keybind| !matches!(keybind.op, Op::ToggleArg(_)))
        .filter(move |keybind| keybind.menu == expected)
}

pub(crate) fn arg_list<'a>(
    bindings: &'a [Binding],
    pending: &Menu,
) -> impl Iterator<Item = &'a Binding> {
    let expected = if pending == &Menu::Help {
        None
    } else {
        Some(*pending)
    };

    bindings
        .iter()
        .filter(|keybind| matches!(keybind.op, Op::ToggleArg(_)))
        .filter(move |keybind| keybind.menu == expected)
}
