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
    pub(crate) args: BTreeMap<Cow<'static, str>, arg::Arg>,
}

impl PendingMenu {
    pub fn init(menu: Menu) -> Self {
        Self {
            menu,
            args: match menu {
                Menu::Root => &[],
                Menu::Branch => ops::checkout::ARGS,
                Menu::Commit => ops::commit::ARGS,
                Menu::Fetch => ops::fetch::ARGS,
                Menu::Help => &[],
                Menu::Log => ops::log::ARGS,
                Menu::Pull => ops::pull::ARGS,
                Menu::Push => ops::push::ARGS,
                Menu::Rebase => ops::rebase::ARGS,
                Menu::Reset => ops::reset::ARGS,
                Menu::Revert => ops::revert::ARGS,
                Menu::Stash => ops::stash::ARGS,
            }
            .iter()
            .map(|arg| (Cow::from(arg.arg), arg.clone()))
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
