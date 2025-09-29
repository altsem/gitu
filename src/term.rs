use crate::{Res, error::Error};
use crossterm::{
    ExecutableCommand,
    event::{EnableMouseCapture, Event},
    terminal::{
        EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
        is_raw_mode_enabled,
    },
};
use ratatui::{
    Terminal,
    backend::{Backend, CrosstermBackend, TestBackend},
    layout::Size,
    prelude::{Position, backend::WindowSize, buffer::Cell},
};
use std::io::{self, Stderr, stderr};
use std::{fmt::Display, time::Duration};

pub type Term = Terminal<TermBackend>;

// TODO It would be more logical if the following top-level functions also were in 'TermBackend'.
//      However left here for now.

pub fn alternate_screen<T, F: Fn() -> Res<T>>(fun: F) -> Res<T> {
    stderr()
        .execute(EnterAlternateScreen)
        .map_err(Error::Term)?;
    let result = fun();
    stderr()
        .execute(LeaveAlternateScreen)
        .map_err(Error::Term)?;
    result
}

pub fn raw_mode<T, F: Fn() -> Res<T>>(fun: F) -> Res<T> {
    let was_raw_mode_enabled = is_raw_mode_enabled().map_err(Error::Term)?;

    if !was_raw_mode_enabled {
        enable_raw_mode().map_err(Error::Term)?;
    }

    let result = fun();

    if !was_raw_mode_enabled {
        disable_raw_mode().map_err(Error::Term)?;
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
    Test {
        backend: TestBackend,
        events: Vec<Event>,
    },
}

impl Backend for TermBackend {
    fn draw<'a, I>(&mut self, content: I) -> io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        match self {
            TermBackend::Crossterm(t) => t.draw(content),
            TermBackend::Test { backend, .. } => backend.draw(content),
        }
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        match self {
            TermBackend::Crossterm(t) => t.hide_cursor(),
            TermBackend::Test { backend, .. } => backend.hide_cursor(),
        }
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        match self {
            TermBackend::Crossterm(t) => t.show_cursor(),
            TermBackend::Test { backend, .. } => backend.show_cursor(),
        }
    }

    fn get_cursor_position(&mut self) -> io::Result<Position> {
        match self {
            TermBackend::Crossterm(t) => t.get_cursor_position(),
            TermBackend::Test { backend, .. } => backend.get_cursor_position(),
        }
    }

    fn set_cursor_position<P: Into<Position>>(&mut self, position: P) -> io::Result<()> {
        match self {
            TermBackend::Crossterm(t) => t.set_cursor_position(position),
            TermBackend::Test { backend, .. } => backend.set_cursor_position(position),
        }
    }

    fn clear(&mut self) -> io::Result<()> {
        match self {
            TermBackend::Crossterm(t) => t.clear(),
            TermBackend::Test { backend, .. } => backend.clear(),
        }
    }

    fn size(&self) -> io::Result<Size> {
        match self {
            TermBackend::Crossterm(t) => t.size(),
            TermBackend::Test { backend, .. } => backend.size(),
        }
    }

    fn window_size(&mut self) -> io::Result<WindowSize> {
        match self {
            TermBackend::Crossterm(t) => t.window_size(),
            TermBackend::Test { backend, .. } => backend.window_size(),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            TermBackend::Crossterm(t) => Backend::flush(t),
            TermBackend::Test { backend, .. } => backend.flush(),
        }
    }
}

impl TermBackend {
    pub fn enter_alternate_screen(&mut self) -> Res<()> {
        match self {
            TermBackend::Crossterm(c) => c
                .execute(EnterAlternateScreen)
                .map_err(Error::Term)
                .map(|_| ()),
            TermBackend::Test { .. } => Ok(()),
        }
    }

    pub fn enable_raw_mode(&self) -> Res<()> {
        match self {
            TermBackend::Crossterm(_) => enable_raw_mode().map_err(Error::Term),
            TermBackend::Test { .. } => Ok(()),
        }
    }

    pub fn disable_raw_mode(&self) -> Res<()> {
        match self {
            TermBackend::Crossterm(_) => disable_raw_mode().map_err(Error::Term),
            TermBackend::Test { .. } => Ok(()),
        }
    }

    pub fn enable_mouse_capture(&mut self) -> Res<()> {
        match self {
            TermBackend::Crossterm(t) => {
                t.execute(EnableMouseCapture).map_err(Error::Term)?;
                Ok(())
            }
            TermBackend::Test { .. } => Ok(()),
        }
    }

    pub fn poll_event(&self, timeout: Duration) -> Res<bool> {
        match self {
            TermBackend::Crossterm(_) => crossterm::event::poll(timeout).map_err(Error::Term),
            TermBackend::Test { events, .. } => {
                if events.is_empty() {
                    Err(Error::NoMoreEvents)
                } else {
                    Ok(true)
                }
            }
        }
    }

    pub fn read_event(&mut self) -> Res<Event> {
        match self {
            TermBackend::Crossterm(_) => crossterm::event::read().map_err(Error::Term),
            TermBackend::Test { events, .. } => events.pop().ok_or(Error::NoMoreEvents),
        }
    }
}
