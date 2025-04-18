use crate::{error::Error, Res};
use ratatui::{
    backend::{Backend, TestBackend},
    layout::Size,
    prelude::{backend::WindowSize, buffer::Cell, Position, TermwizBackend},
    Terminal,
};
use std::{io, time::Duration};
use termwiz::{
    caps::Capabilities,
    input::InputEvent,
    terminal::Terminal as _,
    terminal::{buffered::BufferedTerminal, SystemTerminal},
};

pub type Term = Terminal<TermBackend>;

pub fn create_backend() -> Res<TermBackend> {
    let buffered_terminal = BufferedTerminal::new(
        SystemTerminal::new(Capabilities::new_from_env().map_err(Error::Termwiz)?)
            .map_err(Error::Termwiz)?,
    )
    .map_err(Error::Termwiz)?;

    Ok(TermBackend::Termwiz(
        TermwizBackend::with_buffered_terminal(buffered_terminal),
    ))
}

pub enum TermBackend {
    Termwiz(TermwizBackend),
    #[allow(dead_code)]
    Test {
        backend: TestBackend,
        events: Vec<InputEvent>,
    },
}

impl Backend for TermBackend {
    fn draw<'a, I>(&mut self, content: I) -> io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        match self {
            TermBackend::Termwiz(t) => t.draw(content),
            TermBackend::Test { backend, .. } => backend.draw(content),
        }
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        match self {
            TermBackend::Termwiz(t) => t.hide_cursor(),
            TermBackend::Test { backend, .. } => backend.hide_cursor(),
        }
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        match self {
            TermBackend::Termwiz(t) => t.show_cursor(),
            TermBackend::Test { backend, .. } => backend.show_cursor(),
        }
    }

    fn get_cursor_position(&mut self) -> io::Result<Position> {
        match self {
            TermBackend::Termwiz(t) => t.get_cursor_position(),
            TermBackend::Test { backend, .. } => backend.get_cursor_position(),
        }
    }

    fn set_cursor_position<P: Into<Position>>(&mut self, position: P) -> io::Result<()> {
        match self {
            TermBackend::Termwiz(t) => t.set_cursor_position(position),
            TermBackend::Test { backend, .. } => backend.set_cursor_position(position),
        }
    }

    fn clear(&mut self) -> io::Result<()> {
        match self {
            TermBackend::Termwiz(t) => t.clear(),
            TermBackend::Test { backend, .. } => backend.clear(),
        }
    }

    fn size(&self) -> io::Result<Size> {
        match self {
            TermBackend::Termwiz(t) => t.size(),
            TermBackend::Test { backend, .. } => backend.size(),
        }
    }

    fn window_size(&mut self) -> io::Result<WindowSize> {
        match self {
            TermBackend::Termwiz(t) => t.window_size(),
            TermBackend::Test { backend, .. } => backend.window_size(),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            TermBackend::Termwiz(t) => t.flush(),
            TermBackend::Test { backend, .. } => backend.flush(),
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
            TermBackend::Test { .. } => Ok(()),
        }
    }

    pub fn enable_raw_mode(&mut self) -> Res<()> {
        match self {
            TermBackend::Termwiz(t) => t
                .buffered_terminal_mut()
                .terminal()
                .set_raw_mode()
                .map_err(Error::Termwiz),
            TermBackend::Test { .. } => Ok(()),
        }
    }

    pub fn disable_raw_mode(&mut self) -> Res<()> {
        match self {
            TermBackend::Termwiz(t) => t
                .buffered_terminal_mut()
                .terminal()
                .set_cooked_mode()
                .map_err(Error::Termwiz),
            TermBackend::Test { .. } => Ok(()),
        }
    }

    pub fn poll_input(&mut self, wait: Option<Duration>) -> Res<Option<InputEvent>> {
        match self {
            TermBackend::Termwiz(t) => t
                .buffered_terminal_mut()
                .terminal()
                .poll_input(wait)
                .map_err(Error::Termwiz),
            TermBackend::Test { events, .. } => events.pop().map(Some).ok_or(Error::NoMoreEvents),
        }
    }
}
