use std::borrow::Cow;
use std::collections::BTreeMap;
use std::ffi::OsString;

use serde::{Deserialize, Serialize};

use crate::ops;

#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Menu {
    #[serde(rename = "root")]
    Root,
    #[serde(rename = "branch_menu")]
    Branch,
    #[serde(rename = "commit_menu")]
    Commit,
    #[serde(rename = "fetch_menu")]
    Fetch,
    #[serde(rename = "help_menu")]
    Help,
    #[serde(rename = "log_menu")]
    Log,
    #[serde(rename = "pull_menu")]
    Pull,
    #[serde(rename = "push_menu")]
    Push,
    #[serde(rename = "rebase_menu")]
    Rebase,
    #[serde(rename = "reset_menu")]
    Reset,
    #[serde(rename = "stash_menu")]
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
                Menu::Root => &[],
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
