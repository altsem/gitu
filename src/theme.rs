use ratatui::style::Color;

lazy_static::lazy_static! {
    pub static ref CURRENT_THEME: Theme = BASE16_THEME;
}

pub struct Theme {
    pub section: Color,
    pub unstaged_file: Color,
    pub file: Color,
    pub hunk_header: Color,
    pub highlight: Color,
    pub dim_highlight: Color,
}

pub const BASE16_THEME: Theme = Theme {
    section: Color::Yellow,
    unstaged_file: Color::Red,
    file: Color::Magenta,
    hunk_header: Color::Blue,
    highlight: Color::DarkGray,
    dim_highlight: Color::LightGreen,
};
