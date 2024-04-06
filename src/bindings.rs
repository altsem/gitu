use crate::{key_parser, menu::Menu, ops::Op};
use crossterm::event::{KeyCode, KeyModifiers};

pub(crate) struct Bindings {
    vec: Vec<Binding>,
}

impl Default for Bindings {
    fn default() -> Self {
        Self {
            vec: [
                // Generic
                (None, "q", Op::Quit),
                (None, "<esc>", Op::Quit),
                (None, "g", Op::Refresh),
                // Editor
                (None, "<tab>", Op::ToggleSection),
                (None, "k", Op::MoveUp),
                (None, "p", Op::MoveUp),
                (None, "<up>", Op::MoveUp),
                (None, "j", Op::MoveDown),
                (None, "n", Op::MoveDown),
                (None, "<down>", Op::MoveDown),
                (None, "<ctrl+k>", Op::MoveUpLine),
                (None, "<ctrl+p>", Op::MoveUpLine),
                (None, "<ctrl+up>", Op::MoveUpLine),
                (None, "<ctrl+j>", Op::MoveDownLine),
                (None, "<ctrl+n>", Op::MoveDownLine),
                (None, "<ctrl+down>", Op::MoveDownLine),
                (None, "<ctrl+u>", Op::HalfPageUp),
                (None, "<ctrl+d>", Op::HalfPageDown),
                // Help
                (None, "h", Op::Menu(Menu::Help)),
                (Some(Menu::Help), "q", Op::Quit),
                (Some(Menu::Help), "<esc>", Op::Quit),
                // Branch
                (None, "b", Op::Menu(Menu::Branch)),
                (Some(Menu::Branch), "b", Op::Checkout),
                (Some(Menu::Branch), "c", Op::CheckoutNewBranch),
                (Some(Menu::Branch), "q", Op::Quit),
                (Some(Menu::Branch), "<esc>", Op::Quit),
                // Commit
                (None, "c", Op::Menu(Menu::Commit)),
                (Some(Menu::Commit), "c", Op::Commit),
                (Some(Menu::Commit), "a", Op::CommitAmend),
                (Some(Menu::Commit), "f", Op::CommitFixup),
                (Some(Menu::Commit), "q", Op::Quit),
                (Some(Menu::Commit), "<esc>", Op::Quit),
                // Fetch
                (None, "f", Op::Menu(Menu::Fetch)),
                (Some(Menu::Fetch), "a", Op::FetchAll),
                (Some(Menu::Fetch), "q", Op::Quit),
                (Some(Menu::Fetch), "<esc>", Op::Quit),
                // Log
                (None, "l", Op::Menu(Menu::Log)),
                (Some(Menu::Log), "l", Op::LogCurrent),
                (Some(Menu::Log), "o", Op::LogOther),
                (Some(Menu::Log), "q", Op::Quit),
                (Some(Menu::Log), "<esc>", Op::Quit),
                // Pull
                (None, "F", Op::Menu(Menu::Pull)),
                (Some(Menu::Pull), "p", Op::Pull),
                (Some(Menu::Pull), "q", Op::Quit),
                (Some(Menu::Pull), "<esc>", Op::Quit),
                // Push
                (None, "P", Op::Menu(Menu::Push)),
                (Some(Menu::Push), "-f", Op::ToggleArg("--force-with-lease")),
                (Some(Menu::Push), "p", Op::Push),
                (Some(Menu::Push), "q", Op::Quit),
                (Some(Menu::Push), "<esc>", Op::Quit),
                // Rebase
                (None, "r", Op::Menu(Menu::Rebase)),
                (Some(Menu::Rebase), "i", Op::RebaseInteractive),
                (Some(Menu::Rebase), "a", Op::RebaseAbort),
                (Some(Menu::Rebase), "c", Op::RebaseContinue),
                (Some(Menu::Rebase), "e", Op::RebaseElsewhere),
                (Some(Menu::Rebase), "f", Op::RebaseAutosquash),
                (Some(Menu::Rebase), "q", Op::Quit),
                (Some(Menu::Rebase), "<esc>", Op::Quit),
                // Reset
                (None, "X", Op::Menu(Menu::Reset)),
                (Some(Menu::Reset), "s", Op::ResetSoft),
                (Some(Menu::Reset), "m", Op::ResetMixed),
                (Some(Menu::Reset), "h", Op::ResetHard),
                (Some(Menu::Reset), "q", Op::Quit),
                (Some(Menu::Reset), "<esc>", Op::Quit),
                // Show
                (None, "<enter>", Op::Show),
                // Show refs
                (None, "y", Op::ShowRefs),
                // Stash
                (None, "z", Op::Menu(Menu::Stash)),
                (Some(Menu::Stash), "z", Op::Stash),
                (Some(Menu::Stash), "i", Op::StashIndex),
                (Some(Menu::Stash), "w", Op::StashWorktree),
                (Some(Menu::Stash), "x", Op::StashKeepIndex),
                (Some(Menu::Stash), "p", Op::StashPop),
                (Some(Menu::Stash), "a", Op::StashApply),
                (Some(Menu::Stash), "k", Op::StashDrop),
                (Some(Menu::Stash), "q", Op::Quit),
                (Some(Menu::Stash), "<esc>", Op::Quit),
                // Discard
                (None, "K", Op::Discard),
                // Target actions
                (None, "s", Op::Stage),
                (None, "u", Op::Unstage),
            ]
            .into_iter()
            .map(|(menu, keys, op)| Binding::new(menu, keys, op))
            .collect(),
        }
    }
}

impl Bindings {
    pub(crate) fn match_bindings<'a>(
        &'a self,
        pending: &'a Option<Menu>,
        events: &'a [(KeyModifiers, KeyCode)],
    ) -> impl Iterator<Item = &'a Binding> + 'a {
        self.vec
            .iter()
            .rev()
            // TODO Support multiple keys in a sequence
            .filter(move |binding| &binding.menu == pending)
            .filter(|binding| binding.keys.starts_with(events))
    }

    pub(crate) fn list<'a>(&'a self, pending: &Menu) -> impl Iterator<Item = &'a Binding> {
        let expected = if pending == &Menu::Help {
            None
        } else {
            Some(*pending)
        };

        self.vec
            .iter()
            .filter(|keybind| !matches!(keybind.op, Op::ToggleArg(_)))
            .filter(move |keybind| keybind.menu == expected)
    }

    pub(crate) fn arg_list<'a>(&'a self, pending: &Menu) -> impl Iterator<Item = &'a Binding> {
        let expected = if pending == &Menu::Help {
            None
        } else {
            Some(*pending)
        };

        self.vec
            .iter()
            .filter(|keybind| matches!(keybind.op, Op::ToggleArg(_)))
            .filter(move |keybind| keybind.menu == expected)
    }
}

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
