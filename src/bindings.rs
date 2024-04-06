use std::collections::BTreeMap;

use crate::{
    key_parser,
    menu::{Menu, PendingMenu},
    ops::Op,
};
use crossterm::event::{KeyCode, KeyModifiers};

pub(crate) struct Bindings {
    vec: Vec<Binding>,
}

impl From<&BTreeMap<Menu, BTreeMap<Op, Vec<String>>>> for Bindings {
    fn from(value: &BTreeMap<Menu, BTreeMap<Op, Vec<String>>>) -> Self {
        Self {
            vec: value
                .iter()
                .flat_map(|(menu, ops)| {
                    ops.iter().flat_map(|(op, binds)| {
                        binds
                            .iter()
                            .map(|keys| Binding::new(*menu, keys, op.clone()))
                    })
                })
                .collect(),
        }
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
            // TODO Support multiple keys in a sequence
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
    pub fn new(menu: Menu, raw_keys: &str, op: Op) -> Self {
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
