use crate::{Res, APP_NAME};
use figment::{
    providers::{Format, Toml},
    Figment,
};
use ratatui::style::{Color, Modifier, Style};
use serde::Deserialize;

const DEFAULT_CONFIG: &str = include_str!("default_config.toml");

#[derive(Default, Debug, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub style: StyleConfig,
}

#[derive(Default, Debug, Deserialize)]
pub struct GeneralConfig {
    pub confirm_quit: BoolConfigEntry,
}

#[derive(Default, Debug, Deserialize)]
pub struct BoolConfigEntry {
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Default, Debug, Deserialize)]
pub struct StyleConfig {
    pub section_header: StyleConfigEntry,
    pub file_header: StyleConfigEntry,
    pub hunk_header: StyleConfigEntry,

    pub line_added: StyleConfigEntry,
    pub line_removed: StyleConfigEntry,
    pub line_highlight: LineHighlightConfig,

    pub cursor: StyleConfigEntry,
    pub selection_line: StyleConfigEntry,
    pub selection_bar: StyleConfigEntry,
    pub selection_area: StyleConfigEntry,

    pub hash: StyleConfigEntry,
    pub branch: StyleConfigEntry,
    pub remote: StyleConfigEntry,
    pub tag: StyleConfigEntry,

    pub command: StyleConfigEntry,
    pub hotkey: StyleConfigEntry,
}

#[derive(Default, Debug, Deserialize)]
pub struct LineHighlightConfig {
    #[serde(default)]
    pub changed: StyleConfigEntry,
    #[serde(default)]
    pub unchanged: StyleConfigEntry,
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
        Figment::new()
            .merge(Toml::string(DEFAULT_CONFIG))
            .merge(Toml::file(app_dirs.config_dir().join("config.toml")))
            .extract()?
    } else {
        Config::default()
    };

    Ok(config)
}

pub fn init_test_config() -> Res<Config> {
    Ok(Figment::new()
        .merge(Toml::string(DEFAULT_CONFIG))
        .extract()?)
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
                line_added.bg = "light green"
                "#,
            ))
            .extract()
            .unwrap();

        assert_eq!(config.style.line_added.bg, Some(Color::LightGreen));
        assert_eq!(config.style.line_added.fg, Some(Color::Green));
    }
}
