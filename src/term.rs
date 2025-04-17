use crate::{error::Error, Res};
use ratatui::{
    backend::{Backend, TestBackend},
    layout::Size,
    prelude::{backend::WindowSize, buffer::Cell, Position, TermwizBackend},
    Terminal,
};
use std::io::{self};
use std::{fmt::Display, time::Duration};
use termwiz::{input::InputEvent, terminal::Terminal as _};

pub type Term = Terminal<TermBackend>;

fn print_err<T, E: Display>(result: Result<T, E>) {
    match result {
        Ok(_) => (),
        Err(error) => eprintln!("Error: {}", error),
    };
}

pub fn backend() -> TermBackend {
    // TODO Remove unwrap
    TermBackend::Termwiz(TermwizBackend::new().unwrap())
}

pub enum TermBackend {
    Termwiz(TermwizBackend),
    #[allow(dead_code)]
    Test(TestBackend),
}

impl Backend for TermBackend {
    fn draw<'a, I>(&mut self, content: I) -> io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        match self {
            TermBackend::Termwiz(t) => t.draw(content),
            TermBackend::Test(t) => t.draw(content),
        }
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        match self {
            TermBackend::Termwiz(t) => t.hide_cursor(),
            TermBackend::Test(t) => t.hide_cursor(),
        }
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        match self {
            TermBackend::Termwiz(t) => t.show_cursor(),
            TermBackend::Test(t) => t.show_cursor(),
        }
    }

    fn get_cursor_position(&mut self) -> io::Result<Position> {
        match self {
            TermBackend::Termwiz(t) => t.get_cursor_position(),
            TermBackend::Test(t) => t.get_cursor_position(),
        }
    }

    fn set_cursor_position<P: Into<Position>>(&mut self, position: P) -> io::Result<()> {
        match self {
            TermBackend::Termwiz(t) => t.set_cursor_position(position),
            TermBackend::Test(t) => t.set_cursor_position(position),
        }
    }

    fn clear(&mut self) -> io::Result<()> {
        match self {
            TermBackend::Termwiz(t) => t.clear(),
            TermBackend::Test(t) => t.clear(),
        }
    }

    fn size(&self) -> io::Result<Size> {
        match self {
            TermBackend::Termwiz(t) => t.size(),
            TermBackend::Test(t) => t.size(),
        }
    }

    fn window_size(&mut self) -> io::Result<WindowSize> {
        match self {
            TermBackend::Termwiz(t) => t.window_size(),
            TermBackend::Test(t) => t.window_size(),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            TermBackend::Termwiz(t) => t.flush(),
            TermBackend::Test(t) => t.flush(),
        }
    }
}

impl TermBackend {
    pub fn enter_alternate_screen(&mut self) -> Res<()> {
        match self {
            TermBackend::Termwiz(c) => c
                .buffered_terminal_mut()
                .terminal()
                .enter_alternate_screen()
                .map_err(Error::Termwiz),
            TermBackend::Test(_) => Ok(()),
        }
    }

    pub fn enable_raw_mode(&mut self) -> Res<()> {
        match self {
            TermBackend::Termwiz(t) => t
                .buffered_terminal_mut()
                .terminal()
                .set_raw_mode()
                .map_err(Error::Termwiz),
            TermBackend::Test(_) => Ok(()),
        }
    }

    pub fn disable_raw_mode(&mut self) -> Res<()> {
        match self {
            TermBackend::Termwiz(t) => t
                .buffered_terminal_mut()
                .terminal()
                .set_cooked_mode()
                .map_err(Error::Termwiz),
            TermBackend::Test(_) => Ok(()),
        }
    }

    pub fn poll_input(&mut self, wait: Option<Duration>) -> Res<Option<InputEvent>> {
        match self {
            TermBackend::Termwiz(t) => t
                .buffered_terminal_mut()
                .terminal()
                .poll_input(wait)
                .map_err(Error::Termwiz),
            TermBackend::Test(test_backend) => todo!(),
        }
    }
}
