use std::collections::BTreeMap;

use crate::{
    error::Error,
    key_parser,
    menu::{Menu, PendingMenu},
    ops::Op,
};
use crossterm::event::{KeyCode, KeyModifiers};

pub(crate) struct Bindings {
    vec: Vec<Binding>,
}

// Used to parse the bindings coming out of `figment`. If there are any issues,
// these are collected so the user can be informed.
impl TryFrom<BTreeMap<Menu, BTreeMap<Op, Vec<String>>>> for Bindings {
    type Error = crate::error::Error;

    fn try_from(value: BTreeMap<Menu, BTreeMap<Op, Vec<String>>>) -> Result<Self, Self::Error> {
        let mut bindings = Vec::new();
        let mut bad_bindings = Vec::new();
        for (menu, ops) in value {
            for (op, binds) in ops {
                for keys in binds {
                    if let Some(binding) = Binding::parse(menu, &keys, op.clone()) {
                        bindings.push(binding);
                    } else {
                        // Format bad key bindings like " - {menu}.{op} = {keys}"
                        bad_bindings.push(format!(
                            "- {}.{} = {}",
                            menu.as_ref(),
                            op.as_ref(),
                            keys
                        ));
                    }
                }
            }
        }

        // If any bindings are bad, present them all to the user as an error.
        if !bad_bindings.is_empty() {
            return Err(Error::Bindings {
                bad_key_bindings: bad_bindings,
            });
        }

        Ok(Self { vec: bindings })
    }
}

impl Bindings {
    pub(crate) fn match_bindings<'a>(
        &'a self,
        pending: &'a Menu,
        events: &'a [(KeyModifiers, KeyCode)],
    ) -> impl Iterator<Item = &'a Binding> + 'a {
        self.vec
            .iter()
            .filter(move |binding| &binding.menu == pending)
            .filter(|binding| binding.keys.starts_with(events))
    }

    pub(crate) fn list<'a>(&'a self, pending: &Menu) -> impl Iterator<Item = &'a Binding> {
        let expected = if pending == &Menu::Help {
            Menu::Root
        } else {
            *pending
        };

        self.vec
            .iter()
            .filter(|keybind| !matches!(keybind.op, Op::ToggleArg(_)))
            .filter(move |keybind| keybind.menu == expected)
    }

    pub(crate) fn arg_list<'a>(
        &'a self,
        pending: &'a PendingMenu,
    ) -> impl Iterator<Item = &'a Binding> {
        let expected = if pending.menu == Menu::Help {
            Menu::Root
        } else {
            pending.menu
        };

        self.vec
            .iter()
            .filter(|keybind| {
                if let Op::ToggleArg(ref arg) = keybind.op {
                    pending.args.contains_key(arg.as_str())
                } else {
                    false
                }
            })
            .filter(move |keybind| keybind.menu == expected)
    }
}

pub(crate) struct Binding {
    pub menu: Menu,
    pub raw: String,
    pub keys: Vec<(KeyModifiers, KeyCode)>,
    pub op: Op,
}

impl Binding {
    /// Attempt to parse the key combination passed in. If it fails, `None` is
    /// returned; the caller should report this to the user.
    pub fn parse(menu: Menu, raw_keys: &str, op: Op) -> Option<Self> {
        if let Ok(("", keys)) = key_parser::parse_config_keys(raw_keys) {
            Some(Self {
                menu,
                raw: raw_keys.to_string(),
                keys,
                op,
            })
        } else {
            None
        }
    }
}
