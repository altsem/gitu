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
    pub command: Color,
    pub hotkey: Color,
    pub branch: Color,
    pub remote: Color,
    pub tag: Color,
    pub added: Color,
    pub removed: Color,
    pub oid: Color,
}

pub const DEFAULT_THEME: Theme = {
    let red = Color::Red;
    let green = Color::Green;
    let blue = Color::Blue;
    let yellow = Color::Yellow;
    let magenta = Color::Magenta;

    Theme {
        section: yellow,
        unstaged_file: red,
        unmerged_file: red,
        file: magenta,
        hunk_header: blue,
        command: blue,
        hotkey: magenta,
        branch: green,
        remote: red,
        tag: yellow,
        added: green,
        removed: red,
        oid: yellow,
    }
};
