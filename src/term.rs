use crate::Res;
use crossterm::{
    terminal::{
        disable_raw_mode, enable_raw_mode, is_raw_mode_enabled, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
    ExecutableCommand,
};
use ratatui::{
    backend::{Backend, CrosstermBackend, TestBackend},
    layout::Size,
    prelude::{backend::WindowSize, buffer::Cell, Position},
    Terminal,
};
use std::fmt::Display;
use std::io::{self, stderr, Stderr};

pub type Term = Terminal<TermBackend>;

// TODO It would be more logical if the following top-level functions also were in 'TermBackend'.
//      However left here for now.

pub fn alternate_screen<T, F: Fn() -> Res<T>>(fun: F) -> Res<T> {
    stderr().execute(EnterAlternateScreen)?;
    let result = fun();
    stderr().execute(LeaveAlternateScreen)?;
    result
}

pub fn raw_mode<T, F: Fn() -> Res<T>>(fun: F) -> Res<T> {
    let was_raw_mode_enabled = is_raw_mode_enabled()?;

    if !was_raw_mode_enabled {
        enable_raw_mode()?;
    }

    let result = fun();

    if !was_raw_mode_enabled {
        disable_raw_mode()?;
    }

    result
}

pub fn cleanup_alternate_screen() {
    print_err(stderr().execute(LeaveAlternateScreen));
}

pub fn cleanup_raw_mode() {
    print_err(disable_raw_mode());
}

fn print_err<T, E: Display>(result: Result<T, E>) {
    match result {
        Ok(_) => (),
        Err(error) => eprintln!("Error: {}", error),
    };
}

pub fn backend() -> TermBackend {
    TermBackend::Crossterm(CrosstermBackend::new(stderr()))
}

pub enum TermBackend {
    Crossterm(CrosstermBackend<Stderr>),
    #[allow(dead_code)]
    Test(TestBackend),
}

impl Backend for TermBackend {
    fn draw<'a, I>(&mut self, content: I) -> io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        match self {
            TermBackend::Crossterm(t) => t.draw(content),
            TermBackend::Test(t) => t.draw(content),
        }
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        match self {
            TermBackend::Crossterm(t) => t.hide_cursor(),
            TermBackend::Test(t) => t.hide_cursor(),
        }
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        match self {
            TermBackend::Crossterm(t) => t.show_cursor(),
            TermBackend::Test(t) => t.show_cursor(),
        }
    }

    fn get_cursor_position(&mut self) -> io::Result<Position> {
        match self {
            TermBackend::Crossterm(t) => t.get_cursor_position(),
            TermBackend::Test(t) => t.get_cursor_position(),
        }
    }

    fn set_cursor_position<P: Into<Position>>(&mut self, position: P) -> io::Result<()> {
        match self {
            TermBackend::Crossterm(t) => t.set_cursor_position(position),
            TermBackend::Test(t) => t.set_cursor_position(position),
        }
    }

    fn clear(&mut self) -> io::Result<()> {
        match self {
            TermBackend::Crossterm(t) => t.clear(),
            TermBackend::Test(t) => t.clear(),
        }
    }

    fn size(&self) -> io::Result<Size> {
        match self {
            TermBackend::Crossterm(t) => t.size(),
            TermBackend::Test(t) => t.size(),
        }
    }

    fn window_size(&mut self) -> io::Result<WindowSize> {
        match self {
            TermBackend::Crossterm(t) => t.window_size(),
            TermBackend::Test(t) => t.window_size(),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            TermBackend::Crossterm(t) => t.flush(),
            TermBackend::Test(t) => t.flush(),
        }
    }
}

impl TermBackend {
    pub fn enter_alternate_screen(&mut self) -> io::Result<()> {
        match self {
            TermBackend::Crossterm(c) => c.execute(EnterAlternateScreen).map(|_| ()),
            TermBackend::Test(_) => Ok(()),
        }
    }

    pub fn enable_raw_mode(&self) -> io::Result<()> {
        match self {
            TermBackend::Crossterm(_) => enable_raw_mode(),
            TermBackend::Test(_) => Ok(()),
        }
    }

    pub fn disable_raw_mode(&self) -> io::Result<()> {
        match self {
            TermBackend::Crossterm(_) => disable_raw_mode(),
            TermBackend::Test(_) => Ok(()),
        }
    }
}
