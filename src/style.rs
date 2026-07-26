use serde::{Deserialize, Deserializer, de};
use std::str::FromStr;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Style {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub add_modifier: Modifier,
    pub sub_modifier: Modifier,
}

impl Style {
    pub const fn new() -> Self {
        Style {
            fg: None,
            bg: None,
            add_modifier: Modifier::empty(),
            sub_modifier: Modifier::empty(),
        }
    }

    /// Lays `other` over `self`, with what `other` leaves unset showing through.
    pub fn patch(mut self, other: Style) -> Style {
        self.fg = other.fg.or(self.fg);
        self.bg = other.bg.or(self.bg);

        self.add_modifier.remove(other.sub_modifier);
        self.add_modifier.insert(other.add_modifier);
        self.sub_modifier.remove(other.add_modifier);
        self.sub_modifier.insert(other.sub_modifier);

        self
    }
}

/// The prompt symbols come out of tui-prompts as ratatui spans.
impl From<ratatui::style::Style> for Style {
    fn from(style: ratatui::style::Style) -> Self {
        Style {
            fg: style.fg.map(Color::from),
            bg: style.bg.map(Color::from),
            add_modifier: Modifier(style.add_modifier.bits()),
            sub_modifier: Modifier(style.sub_modifier.bits()),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    #[default]
    Reset,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    Gray,
    DarkGray,
    LightRed,
    LightGreen,
    LightYellow,
    LightBlue,
    LightMagenta,
    LightCyan,
    White,
    Rgb(u8, u8, u8),
    Indexed(u8),
}

impl From<Color> for crossterm::style::Color {
    fn from(color: Color) -> Self {
        match color {
            Color::Reset => Self::Reset,
            Color::Black => Self::Black,
            Color::Red => Self::DarkRed,
            Color::Green => Self::DarkGreen,
            Color::Yellow => Self::DarkYellow,
            Color::Blue => Self::DarkBlue,
            Color::Magenta => Self::DarkMagenta,
            Color::Cyan => Self::DarkCyan,
            Color::Gray => Self::Grey,
            Color::DarkGray => Self::DarkGrey,
            Color::LightRed => Self::Red,
            Color::LightGreen => Self::Green,
            Color::LightYellow => Self::Yellow,
            Color::LightBlue => Self::Blue,
            Color::LightMagenta => Self::Magenta,
            Color::LightCyan => Self::Cyan,
            Color::White => Self::White,
            Color::Rgb(r, g, b) => Self::Rgb { r, g, b },
            Color::Indexed(i) => Self::AnsiValue(i),
        }
    }
}

impl From<ratatui::style::Color> for Color {
    fn from(color: ratatui::style::Color) -> Self {
        use ratatui::style::Color as R;

        match color {
            R::Reset => Self::Reset,
            R::Black => Self::Black,
            R::Red => Self::Red,
            R::Green => Self::Green,
            R::Yellow => Self::Yellow,
            R::Blue => Self::Blue,
            R::Magenta => Self::Magenta,
            R::Cyan => Self::Cyan,
            R::Gray => Self::Gray,
            R::DarkGray => Self::DarkGray,
            R::LightRed => Self::LightRed,
            R::LightGreen => Self::LightGreen,
            R::LightYellow => Self::LightYellow,
            R::LightBlue => Self::LightBlue,
            R::LightMagenta => Self::LightMagenta,
            R::LightCyan => Self::LightCyan,
            R::White => Self::White,
            R::Rgb(r, g, b) => Self::Rgb(r, g, b),
            R::Indexed(i) => Self::Indexed(i),
        }
    }
}

impl FromStr for Color {
    type Err = ParseColorError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Colors are written by hand in the config, so accept the spellings
        // that are in the wild rather than one canonical form.
        let name = s
            .to_lowercase()
            .replace([' ', '-', '_'], "")
            .replace("bright", "light")
            .replace("grey", "gray")
            .replace("silver", "gray")
            .replace("lightblack", "darkgray")
            .replace("lightwhite", "white")
            .replace("lightgray", "white");

        Ok(match name.as_str() {
            "reset" => Self::Reset,
            "black" => Self::Black,
            "red" => Self::Red,
            "green" => Self::Green,
            "yellow" => Self::Yellow,
            "blue" => Self::Blue,
            "magenta" => Self::Magenta,
            "cyan" => Self::Cyan,
            "gray" => Self::Gray,
            "darkgray" => Self::DarkGray,
            "lightred" => Self::LightRed,
            "lightgreen" => Self::LightGreen,
            "lightyellow" => Self::LightYellow,
            "lightblue" => Self::LightBlue,
            "lightmagenta" => Self::LightMagenta,
            "lightcyan" => Self::LightCyan,
            "white" => Self::White,
            _ => {
                if let Ok(index) = s.parse::<u8>() {
                    Self::Indexed(index)
                } else if let Some((r, g, b)) = parse_hex_color(s) {
                    Self::Rgb(r, g, b)
                } else {
                    return Err(ParseColorError);
                }
            }
        })
    }
}

fn parse_hex_color(s: &str) -> Option<(u8, u8, u8)> {
    let hex = s.strip_prefix('#')?;
    if hex.len() != 6 {
        return None;
    }

    // Indexing would panic where a multi-byte char straddles the split.
    let channel = |range: std::ops::Range<usize>| u8::from_str_radix(hex.get(range)?, 16).ok();
    Some((channel(0..2)?, channel(2..4)?, channel(4..6)?))
}

#[derive(Debug)]
pub struct ParseColorError;

impl std::fmt::Display for ParseColorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("invalid color")
    }
}

impl std::error::Error for ParseColorError {}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Color::from_str(&s).map_err(|_| de::Error::custom(format!("unknown color: {s}")))
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Modifier(pub u16);

impl Modifier {
    pub const BOLD: Self = Modifier(0b0000_0000_0001);
    pub const DIM: Self = Modifier(0b0000_0000_0010);
    pub const ITALIC: Self = Modifier(0b0000_0000_0100);
    pub const UNDERLINED: Self = Modifier(0b0000_0000_1000);
    pub const SLOW_BLINK: Self = Modifier(0b0000_0001_0000);
    pub const RAPID_BLINK: Self = Modifier(0b0000_0010_0000);
    pub const REVERSED: Self = Modifier(0b0000_0100_0000);
    pub const HIDDEN: Self = Modifier(0b0000_1000_0000);
    pub const CROSSED_OUT: Self = Modifier(0b0001_0000_0000);

    const NAMED: &'static [(&'static str, Self)] = &[
        ("BOLD", Self::BOLD),
        ("DIM", Self::DIM),
        ("ITALIC", Self::ITALIC),
        ("UNDERLINED", Self::UNDERLINED),
        ("SLOW_BLINK", Self::SLOW_BLINK),
        ("RAPID_BLINK", Self::RAPID_BLINK),
        ("REVERSED", Self::REVERSED),
        ("HIDDEN", Self::HIDDEN),
        ("CROSSED_OUT", Self::CROSSED_OUT),
    ];

    pub const fn empty() -> Self {
        Modifier(0)
    }

    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    pub fn insert(&mut self, other: Self) {
        self.0 |= other.0;
    }

    pub fn remove(&mut self, other: Self) {
        self.0 &= !other.0;
    }
}

impl FromStr for Modifier {
    type Err = ParseModifierError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut modifier = Modifier::empty();

        if s.trim().is_empty() {
            return Ok(modifier);
        }

        for name in s.split('|') {
            let name = name.trim();
            let (_, flag) = Modifier::NAMED
                .iter()
                .find(|(named, _)| named.eq_ignore_ascii_case(name))
                .ok_or(ParseModifierError)?;

            modifier.insert(*flag);
        }

        Ok(modifier)
    }
}

#[derive(Debug)]
pub struct ParseModifierError;

impl std::fmt::Display for ParseModifierError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("invalid modifier")
    }
}

impl std::error::Error for ParseModifierError {}

impl<'de> Deserialize<'de> for Modifier {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Modifier::from_str(&s).map_err(|_| de::Error::custom(format!("unknown modifier: {s}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_modifiers() {
        assert_eq!(Modifier::from_str("").unwrap(), Modifier::empty());
        assert_eq!(Modifier::from_str("BOLD").unwrap(), Modifier::BOLD);

        let mut both = Modifier::BOLD;
        both.insert(Modifier::ITALIC);
        assert_eq!(Modifier::from_str("BOLD|ITALIC").unwrap(), both);
        assert_eq!(Modifier::from_str("BOLD | ITALIC").unwrap(), both);

        assert!(Modifier::from_str("NONSENSE").is_err());
    }

    #[test]
    fn parse_colors() {
        assert_eq!(Color::from_str("light blue").unwrap(), Color::LightBlue);
        assert_eq!(Color::from_str("LightBlue").unwrap(), Color::LightBlue);
        assert_eq!(Color::from_str("bright-blue").unwrap(), Color::LightBlue);
        assert_eq!(Color::from_str("grey").unwrap(), Color::Gray);
        assert_eq!(Color::from_str("reset").unwrap(), Color::Reset);
        assert_eq!(Color::from_str("255").unwrap(), Color::Indexed(255));
        assert_eq!(
            Color::from_str("#707070").unwrap(),
            Color::Rgb(112, 112, 112)
        );

        assert!(Color::from_str("#70707").is_err());
        assert!(Color::from_str("#€abc").is_err());
        assert!(Color::from_str("nonsense").is_err());
    }
}
