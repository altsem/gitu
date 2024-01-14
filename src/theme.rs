use ratatui::style::Color;

lazy_static::lazy_static! {
    pub static ref CURRENT_THEME: Theme = DEFAULT_THEME;
}

pub struct Theme {
    pub section: Color,
    pub unstaged_file: Color,
    pub unmerged_file: Color,
    pub file: Color,
    pub hunk_header: Color,
    pub highlight: Color,
    pub dim_highlight: Color,
    pub command: Color,
}

// https://github.com/sainnhe/gruvbox-material-vscode
pub const DEFAULT_THEME: Theme = Theme {
    section: Color::Rgb(0xd8, 0xa6, 0x57),
    unstaged_file: Color::Rgb(0xea, 0x69, 0x62),
    unmerged_file: Color::Rgb(0xea, 0x69, 0x62),
    file: Color::Rgb(0xd3, 0x86, 0x9b),
    hunk_header: Color::Rgb(0x7d, 0xae, 0xa3),
    highlight: Color::Rgb(0x50, 0x49, 0x45),
    dim_highlight: Color::Rgb(0x2a, 0x28, 0x27),
    command: Color::Rgb(0x7d, 0xae, 0xa3),
};

pub const BASE16_THEME: Theme = Theme {
    section: Color::Yellow,
    unstaged_file: Color::Red,
    unmerged_file: Color::Red,
    file: Color::Magenta,
    hunk_header: Color::Blue,
    highlight: Color::DarkGray,
    dim_highlight: Color::LightGreen,
    command: Color::Blue,
};
