use std::borrow::Cow;
use std::collections::BTreeMap;
use std::ffi::OsString;

use serde::{Deserialize, Serialize};

use crate::ops;

pub(crate) mod arg;

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
    #[serde(rename = "revert_menu")]
    Revert,
    #[serde(rename = "stash_menu")]
    Stash,
}

pub(crate) struct PendingMenu {
    pub menu: Menu,
    pub is_hidden: bool,
    pub(crate) args: BTreeMap<Cow<'static, str>, arg::Arg>,
}

impl PendingMenu {
    pub fn init(menu: Menu) -> Self {
        Self {
            menu,
            is_hidden: false,
            args: match menu {
                Menu::Root => vec![],
                Menu::Branch => ops::branch::init_args(),
                Menu::Commit => ops::commit::init_args(),
                Menu::Fetch => ops::fetch::init_args(),
                Menu::Help => vec![],
                Menu::Log => ops::log::init_args(),
                Menu::Pull => ops::pull::init_args(),
                Menu::Push => ops::push::init_args(),
                Menu::Rebase => ops::rebase::init_args(),
                Menu::Reset => ops::reset::init_args(),
                Menu::Revert => ops::revert::init_args(),
                Menu::Stash => ops::stash::init_args(),
            }
            .into_iter()
            .map(|arg| (Cow::from(arg.arg), arg))
            .collect(),
        }
    }

    pub fn args(&self) -> Vec<OsString> {
        self.args
            .iter()
            .filter(|&(_k, arg)| arg.is_active())
            .map(|(_, v)| v.get_cli_token().into())
            .collect()
    }
}
