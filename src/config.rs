use std::{collections::BTreeMap, path::PathBuf};

use crate::{Bindings, Res, error::Error, key_parser, menu::Menu, ops::Op};
use crossterm::event::{KeyCode, KeyModifiers};
use etcetera::{BaseStrategy, choose_base_strategy};
use figment::{
    Figment,
    providers::{Format, Toml},
};
use ratatui::style::{Color, Modifier, Style};
use serde::Deserialize;

const DEFAULT_CONFIG: &str = include_str!("default_config.toml");

pub struct Config {
    pub general: GeneralConfig,
    pub style: StyleConfig,
    pub bindings: Bindings,
    pub picker_bindings: PickerBindings,
}

#[derive(Default, Deserialize)]
pub(crate) struct PickerBindingsConfig {
    #[serde(default)]
    pub next: Vec<String>,
    #[serde(default)]
    pub previous: Vec<String>,
    #[serde(default)]
    pub done: Vec<String>,
    #[serde(default)]
    pub cancel: Vec<String>,
}

#[derive(Default, Deserialize)]
pub(crate) struct BindingsConfig {
    #[serde(flatten)]
    pub menus: BTreeMap<Menu, BTreeMap<Op, Vec<String>>>,
    #[serde(default)]
    pub picker: PickerBindingsConfig,
}

#[derive(Default, Deserialize)]
/// Only used to deserialise configurations with `figment`. This should be
/// parsed to be turned into a useful [`Config`].
pub(crate) struct FigmentConfig {
    pub general: GeneralConfig,
    pub style: StyleConfig,
    pub bindings: BindingsConfig,
}

#[derive(Default, Debug, Deserialize)]
pub struct GeneralConfig {
    pub always_show_help: BoolConfigEntry,
    pub confirm_quit: BoolConfigEntry,
    pub refresh_on_file_change: BoolConfigEntry,
    pub confirm_discard: ConfirmDiscardOption,
    pub collapsed_sections: Vec<String>,
    pub stash_list_limit: usize,
    pub recent_commits_limit: usize,
    pub mouse_support: bool,
    pub mouse_scroll_lines: usize,
}

#[derive(Default, Debug, Deserialize)]
pub struct BoolConfigEntry {
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Default, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum ConfirmDiscardOption {
    #[default]
    Line,
    Hunk,
    File,
    Never,
}

#[derive(Default, Debug, Deserialize)]
pub struct StyleConfig {
    pub separator: StyleConfigEntry,

    pub info_msg: StyleConfigEntry,
    pub error_msg: StyleConfigEntry,
    pub command: StyleConfigEntry,

    #[serde(default)]
    pub menu: MenuStyleConfig,

    pub prompt: StyleConfigEntry,

    pub section_header: StyleConfigEntry,
    pub file_header: StyleConfigEntry,
    pub hunk_header: StyleConfigEntry,

    #[serde(default)]
    pub diff_highlight: DiffHighlightConfig,

    #[serde(default)]
    pub syntax_highlight: SyntaxHighlightConfig,

    #[serde(default)]
    pub picker: PickerStyleConfig,

    pub cursor: SymbolStyleConfigEntry,
    pub selection_bar: SymbolStyleConfigEntry,
    pub selection_line: StyleConfigEntry,
    pub selection_area: StyleConfigEntry,

    pub hash: StyleConfigEntry,
    pub branch: StyleConfigEntry,
    pub remote: StyleConfigEntry,
    pub tag: StyleConfigEntry,
}

#[derive(Default, Debug, Deserialize)]
pub struct MenuStyleConfig {
    #[serde(default)]
    pub heading: StyleConfigEntry,
    #[serde(default)]
    pub key: StyleConfigEntry,
    /// Active argument value display (e.g., "--interactive")
    #[serde(default)]
    pub active_arg: StyleConfigEntry,
    /// Inactive argument value display
    #[serde(default)]
    pub inactive_arg: StyleConfigEntry,
}

#[derive(Default, Debug, Deserialize)]
pub struct DiffHighlightConfig {
    #[serde(default)]
    pub tag_old: StyleConfigEntry,
    #[serde(default)]
    pub tag_new: StyleConfigEntry,
    #[serde(default)]
    pub unchanged_old: StyleConfigEntry,
    #[serde(default)]
    pub unchanged_new: StyleConfigEntry,
    #[serde(default)]
    pub changed_old: StyleConfigEntry,
    #[serde(default)]
    pub changed_new: StyleConfigEntry,
}

#[derive(Default, Debug, Deserialize)]
pub struct SyntaxHighlightConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub attribute: StyleConfigEntry,
    #[serde(default)]
    pub comment: StyleConfigEntry,
    #[serde(default)]
    pub constant_builtin: StyleConfigEntry,
    #[serde(default)]
    pub constant: StyleConfigEntry,
    #[serde(default)]
    pub constructor: StyleConfigEntry,
    #[serde(default)]
    pub embedded: StyleConfigEntry,
    #[serde(default)]
    pub function_builtin: StyleConfigEntry,
    #[serde(default)]
    pub function: StyleConfigEntry,
    #[serde(default)]
    pub keyword: StyleConfigEntry,
    #[serde(default)]
    pub number: StyleConfigEntry,
    #[serde(default)]
    pub module: StyleConfigEntry,
    #[serde(default)]
    pub property: StyleConfigEntry,
    #[serde(default)]
    pub operator: StyleConfigEntry,
    #[serde(default)]
    pub punctuation_bracket: StyleConfigEntry,
    #[serde(default)]
    pub punctuation_delimiter: StyleConfigEntry,
    #[serde(default)]
    pub string_special: StyleConfigEntry,
    #[serde(default)]
    pub string: StyleConfigEntry,
    #[serde(default)]
    pub tag: StyleConfigEntry,
    #[serde(default)]
    #[serde(rename = "type")]
    pub type_regular: StyleConfigEntry,
    #[serde(default)]
    pub type_builtin: StyleConfigEntry,
    #[serde(default)]
    pub variable_builtin: StyleConfigEntry,
    #[serde(default)]
    pub variable_parameter: StyleConfigEntry,
}

#[derive(Default, Debug, Deserialize)]
pub struct PickerStyleConfig {
    #[serde(default)]
    pub prompt: StyleConfigEntry,
    #[serde(default)]
    pub info: StyleConfigEntry,
    #[serde(default)]
    pub selection_line: StyleConfigEntry,
    #[serde(default)]
    pub matched: StyleConfigEntry,
}

#[derive(Default, Debug, Deserialize)]
pub struct StyleConfigEntry {
    #[serde(default)]
    fg: Option<Color>,
    #[serde(default)]
    bg: Option<Color>,
    #[serde(default)]
    mods: Option<Modifier>,
}

#[derive(Default, Debug, Deserialize)]
pub struct SymbolStyleConfigEntry {
    #[serde(default)]
    pub symbol: char,
    #[serde(default)]
    fg: Option<Color>,
    #[serde(default)]
    bg: Option<Color>,
    #[serde(default)]
    mods: Option<Modifier>,
}

impl From<&StyleConfigEntry> for Style {
    fn from(val: &StyleConfigEntry) -> Self {
        Style {
            fg: val.fg,
            bg: val.bg,
            underline_color: None,
            add_modifier: val.mods.unwrap_or(Modifier::empty()),
            sub_modifier: Modifier::empty(),
        }
    }
}

impl From<&SymbolStyleConfigEntry> for Style {
    fn from(val: &SymbolStyleConfigEntry) -> Self {
        Style {
            fg: val.fg,
            bg: val.bg,
            underline_color: None,
            add_modifier: val.mods.unwrap_or(Modifier::empty()),
            sub_modifier: Modifier::empty(),
        }
    }
}

pub struct PickerBindings {
    pub next: Vec<Vec<(KeyModifiers, KeyCode)>>,
    pub previous: Vec<Vec<(KeyModifiers, KeyCode)>>,
    pub done: Vec<Vec<(KeyModifiers, KeyCode)>>,
    pub cancel: Vec<Vec<(KeyModifiers, KeyCode)>>,
}

impl TryFrom<PickerBindingsConfig> for PickerBindings {
    type Error = crate::error::Error;

    fn try_from(config: PickerBindingsConfig) -> Result<Self, Self::Error> {
        let mut bad_bindings = Vec::new();

        let next = parse_picker_keys(&config.next, "picker.next", &mut bad_bindings);
        let previous = parse_picker_keys(&config.previous, "picker.previous", &mut bad_bindings);
        let done = parse_picker_keys(&config.done, "picker.done", &mut bad_bindings);
        let cancel = parse_picker_keys(&config.cancel, "picker.cancel", &mut bad_bindings);

        if !bad_bindings.is_empty() {
            return Err(Error::Bindings {
                bad_key_bindings: bad_bindings,
            });
        }

        Ok(Self {
            next,
            previous,
            done,
            cancel,
        })
    }
}

fn parse_picker_keys(
    raw_keys: &[String],
    action_name: &str,
    bad_bindings: &mut Vec<String>,
) -> Vec<Vec<(KeyModifiers, KeyCode)>> {
    raw_keys
        .iter()
        .filter_map(|keys| {
            if let Ok(("", parsed)) = key_parser::parse_config_keys(keys) {
                Some(parsed)
            } else {
                bad_bindings.push(format!("- {} = {}", action_name, keys));
                None
            }
        })
        .collect()
}

pub fn init_config(path: Option<PathBuf>) -> Res<Config> {
    let config_path = path.unwrap_or_else(config_path);

    if config_path.exists() {
        log::info!("Loading config file at {config_path:?}");
    } else {
        log::info!("No config file at {config_path:?}");
    }

    let FigmentConfig {
        general,
        style,
        bindings: bindings_config,
    } = Figment::new()
        .merge(Toml::string(DEFAULT_CONFIG))
        .merge(Toml::file(config_path))
        .extract()
        .map_err(Box::new)
        .map_err(Error::Config)?;
    let bindings = Bindings::try_from(bindings_config.menus)?;
    let picker_bindings = PickerBindings::try_from(bindings_config.picker)?;

    Ok(Config {
        general,
        style,
        bindings,
        picker_bindings,
    })
}

pub fn config_path() -> PathBuf {
    choose_base_strategy()
        .expect("Unable to find the config directory!")
        .config_dir()
        .join("gitu/config.toml")
}

#[cfg(test)]
pub(crate) fn init_test_config() -> Res<Config> {
    let FigmentConfig {
        mut general,
        style,
        bindings: bindings_config,
    } = Figment::new()
        .merge(Toml::string(DEFAULT_CONFIG))
        .extract()
        .map_err(Box::new)
        .map_err(Error::Config)?;

    general.always_show_help.enabled = false;
    general.refresh_on_file_change.enabled = false;

    Ok(Config {
        general,
        style,
        bindings: Bindings::try_from(bindings_config.menus).unwrap(),
        picker_bindings: PickerBindings::try_from(bindings_config.picker).unwrap(),
    })
}

#[cfg(test)]
mod tests {
    use figment::{
        Figment,
        providers::{Format, Toml},
    };
    use ratatui::style::Color;

    use super::{DEFAULT_CONFIG, FigmentConfig};

    #[test]
    fn config_merges() {
        let config: FigmentConfig = Figment::new()
            .merge(Toml::string(DEFAULT_CONFIG))
            .merge(Toml::string(
                r#"
                [style]
                hunk_header.bg = "light green"
                "#,
            ))
            .extract()
            .unwrap();

        assert_eq!(config.style.hunk_header.bg, Some(Color::LightGreen));
        assert_eq!(config.style.hunk_header.fg, Some(Color::Blue));
    }
}
