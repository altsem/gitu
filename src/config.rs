use std::{collections::BTreeMap, path::PathBuf};

use crate::{error::Error, menu::Menu, ops::Op, Res};
use etcetera::{choose_base_strategy, BaseStrategy};
use figment::{
    providers::{Format, Toml},
    Figment,
};
use ratatui::style::{Color, Modifier, Style};
use serde::Deserialize;

const DEFAULT_CONFIG: &str = include_str!("default_config.toml");

#[derive(Default, Debug, Deserialize)]
pub(crate) struct Config {
    pub general: GeneralConfig,
    pub style: StyleConfig,
    pub bindings: BTreeMap<Menu, BTreeMap<Op, Vec<String>>>,
}

#[derive(Default, Debug, Deserialize)]
pub struct GeneralConfig {
    pub always_show_help: BoolConfigEntry,
    pub confirm_quit: BoolConfigEntry,
    pub refresh_on_file_change: BoolConfigEntry,
    pub confirm_discard: ConfirmDiscardOption,
    pub collapsed_sections: Vec<String>,
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
    pub section_header: StyleConfigEntry,
    pub file_header: StyleConfigEntry,
    pub hunk_header: StyleConfigEntry,

    #[serde(default)]
    pub diff_highlight: DiffHighlightConfig,

    #[serde(default)]
    pub syntax_highlight: SyntaxHighlightConfig,

    pub cursor: SymbolStyleConfigEntry,
    pub selection_line: StyleConfigEntry,
    pub selection_bar: SymbolStyleConfigEntry,
    pub selection_area: StyleConfigEntry,

    pub hash: StyleConfigEntry,
    pub branch: StyleConfigEntry,
    pub remote: StyleConfigEntry,
    pub tag: StyleConfigEntry,

    pub command: StyleConfigEntry,
    pub active_arg: StyleConfigEntry,
    pub hotkey: StyleConfigEntry,
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

pub(crate) fn init_config() -> Res<Config> {
    let config_path = config_path();

    if config_path.exists() {
        log::info!("Loading config file at {:?}", config_path);
    } else {
        log::info!("No config file at {:?}", config_path);
    }

    let config = Figment::new()
        .merge(Toml::string(DEFAULT_CONFIG))
        .merge(Toml::file(config_path))
        .extract()
        .map_err(Error::Config)?;

    Ok(config)
}

pub fn config_path() -> PathBuf {
    choose_base_strategy()
        .expect("Unable to find the config directory!")
        .config_dir()
        .join("gitu/config.toml")
}

#[cfg(test)]
pub(crate) fn init_test_config() -> Res<Config> {
    let mut config: Config = Figment::new()
        .merge(Toml::string(DEFAULT_CONFIG))
        .extract()
        .map_err(Error::Config)?;

    config.general.always_show_help.enabled = false;
    config.general.refresh_on_file_change.enabled = false;

    Ok(config)
}

#[cfg(test)]
mod tests {
    use figment::{
        providers::{Format, Toml},
        Figment,
    };
    use ratatui::style::Color;

    use super::{Config, DEFAULT_CONFIG};

    #[test]
    fn config_merges() {
        let config: Config = Figment::new()
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
