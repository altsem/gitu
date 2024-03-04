use crate::{Res, APP_NAME};
use ratatui::style::{Color, Modifier, Style};
use serde::Deserialize;
use std::{fs, io};

const DEFAULT_CONFIG: &str = include_str!("default_config.toml");

#[derive(Debug, Deserialize)]
pub struct Config {
    pub style: StyleConfig,
}

#[derive(Debug, Deserialize)]
pub struct StyleConfig {
    #[serde(default)]
    pub section_header: StyleConfigEntry,
    #[serde(default)]
    pub file_header: StyleConfigEntry,
    #[serde(default)]
    pub hunk_header: StyleConfigEntry,

    #[serde(default)]
    pub line_added: StyleConfigEntry,
    #[serde(default)]
    pub line_removed: StyleConfigEntry,

    #[serde(default)]
    pub selection_line: StyleConfigEntry,
    #[serde(default)]
    pub selection_bar: StyleConfigEntry,
    #[serde(default)]
    pub selection_area: StyleConfigEntry,

    #[serde(default)]
    pub hash: StyleConfigEntry,
    #[serde(default)]
    pub branch: StyleConfigEntry,
    #[serde(default)]
    pub remote: StyleConfigEntry,
    #[serde(default)]
    pub tag: StyleConfigEntry,

    #[serde(default)]
    pub command: StyleConfigEntry,
    #[serde(default)]
    pub hotkey: StyleConfigEntry,
}

impl Default for Config {
    fn default() -> Self {
        toml::from_str(DEFAULT_CONFIG).expect("Failed to parse default_config.toml")
    }
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

pub(crate) fn init_config() -> Res<Config> {
    let config = if let Some(app_dirs) = directories::ProjectDirs::from("", "", APP_NAME) {
        let path = app_dirs.config_dir().join("config.toml");

        match fs::read_to_string(&path) {
            Ok(content) => toml::from_str(&content)?,
            Err(err) => match err.kind() {
                io::ErrorKind::NotFound => Config::default(),
                reason => {
                    log::error!("Error reading config file {:?} {:?}", &path, reason);
                    Config::default()
                }
            },
        }
    } else {
        Config::default()
    };

    Ok(config)
}
