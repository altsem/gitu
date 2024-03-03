use crate::{Res, APP_NAME};
use ratatui::style::{Color, Modifier, Style};
use serde::Deserialize;
use std::{fs, io};

const DEFAULT_CONFIG: &str = include_str!("default_config.toml");

#[derive(Debug, Deserialize)]
pub struct Config {
    pub color: ColorConfig,
}

#[derive(Debug, Deserialize)]
pub struct ColorConfig {
    #[serde(default)]
    pub section: StyleConfig,
    #[serde(default)]
    pub unstaged_file: StyleConfig,
    #[serde(default)]
    pub unmerged_file: StyleConfig,
    #[serde(default)]
    pub file: StyleConfig,
    #[serde(default)]
    pub hunk_header: StyleConfig,
    #[serde(default)]
    pub command: StyleConfig,
    #[serde(default)]
    pub hotkey: StyleConfig,
    #[serde(default)]
    pub branch: StyleConfig,
    #[serde(default)]
    pub remote: StyleConfig,
    #[serde(default)]
    pub tag: StyleConfig,
    #[serde(default)]
    pub added: StyleConfig,
    #[serde(default)]
    pub removed: StyleConfig,
    #[serde(default)]
    pub oid: StyleConfig,
    #[serde(default)]
    pub cursor_line: StyleConfig,
    #[serde(default)]
    pub cursor_section: StyleConfig,
}

impl Default for Config {
    fn default() -> Self {
        toml::from_str(DEFAULT_CONFIG).expect("Failed to parse default_config.toml")
    }
}

#[derive(Default, Debug, Deserialize)]
pub struct StyleConfig {
    #[serde(default)]
    fg: Option<Color>,
    #[serde(default)]
    bg: Option<Color>,
    #[serde(default)]
    mods: Option<Modifier>,
}

impl From<&StyleConfig> for Style {
    fn from(val: &StyleConfig) -> Self {
        Style {
            fg: val.fg,
            bg: val.bg,
            underline_color: None,
            add_modifier: val.mods.unwrap_or(Modifier::empty()),
            sub_modifier: Modifier::empty(),
        }
    }
}

pub(crate) fn load_or_default() -> Res<Config> {
    let config = if let Some(app_dirs) = directories::ProjectDirs::from("", "", APP_NAME) {
        // TODO Write the config when we're happy with the format
        // fs::create_dir_all(app_dirs.config_dir())?;
        let path = app_dirs.config_dir().join("config.toml");

        match fs::read_to_string(&path) {
            Ok(content) => toml::from_str(&content)?,
            Err(err) => match err.kind() {
                io::ErrorKind::NotFound => {
                    // TODO Write the config when we're happy with the format
                    // TODO write_new_config(&path)?;
                    Config::default()
                }
                reason => {
                    eprintln!("Error reading config file {:?} {:?}", &path, reason);
                    Config::default()
                }
            },
        }
    } else {
        Config::default()
    };

    Ok(config)
}

// TODO
// fn write_new_config(path: &std::path::PathBuf) -> Res<()> {
//     if let Err(err) = fs::write(path, DEFAULT_CONFIG) {
//         eprintln!("Error writing config file {:?} {:?}", &path, err);
//     }
//     Ok(())
// }
