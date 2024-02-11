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
    pub hotkey: Color,
    pub branch: Color,
    pub remote: Color,
    pub added: Color,
    pub removed: Color,
}

// https://github.com/sainnhe/gruvbox-material-vscode
pub const DEFAULT_THEME: Theme = {
    let red = Color::Rgb(0xea, 0x69, 0x62);
    let green = Color::Rgb(0xa9, 0xb6, 0x65);
    let blue = Color::Rgb(0x7d, 0xae, 0xa3);
    let yellow = Color::Rgb(0xd8, 0xa6, 0x57);
    let magenta = Color::Rgb(0xd3, 0x86, 0x9b);
    let gray = Color::Rgb(0x50, 0x49, 0x45);
    let dark_gray = Color::Rgb(0x2a, 0x28, 0x27);

    Theme {
        section: yellow,
        unstaged_file: red,
        unmerged_file: red,
        file: magenta,
        hunk_header: blue,
        highlight: gray,
        dim_highlight: dark_gray,
        command: blue,
        hotkey: magenta,
        branch: green,
        remote: red,
        added: green,
        removed: red,
    }
};
