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
    Theme {
        section: Color::Yellow,
        unstaged_file: Color::Red,
        unmerged_file: Color::Red,
        file: Color::Magenta,
        hunk_header: Color::Blue,
        command: Color::Blue,
        hotkey: Color::Magenta,
        branch: Color::Green,
        remote: Color::Red,
        tag: Color::Yellow,
        added: Color::Green,
        removed: Color::Red,
        oid: Color::Yellow,
    }
};
