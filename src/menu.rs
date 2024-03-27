use std::borrow::Cow;
use std::collections::BTreeMap;
use std::ffi::OsString;

use crate::ops;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Menu {
    Branch,
    Commit,
    Fetch,
    Help,
    Log,
    Pull,
    Push,
    Rebase,
    Reset,
    Stash,
}

pub(crate) struct PendingMenu {
    pub menu: Menu,
    pub(crate) args: BTreeMap<Cow<'static, str>, bool>,
}

impl PendingMenu {
    pub fn init(menu: Menu) -> Self {
        Self {
            menu,
            args: match menu {
                Menu::Branch => ops::checkout::args(),
                Menu::Commit => ops::commit::args(),
                Menu::Fetch => ops::fetch::args(),
                Menu::Help => &[],
                Menu::Log => ops::log::args(),
                Menu::Pull => ops::pull::args(),
                Menu::Push => ops::push::args(),
                Menu::Rebase => ops::rebase::args(),
                Menu::Reset => ops::reset::args(),
                Menu::Stash => ops::stash::args(),
            }
            .iter()
            .map(|&(k, v)| (Cow::from(k), v))
            .collect(),
        }
    }

    pub fn args(&self) -> Vec<OsString> {
        self.args
            .iter()
            .filter(|&(_k, &v)| v)
            .map(|(k, _v)| k.to_string().into())
            .collect()
    }
}
