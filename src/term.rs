use crate::{Res, config::Config, error::Error};
use crossterm::{
    QueueableCommand,
    cursor::{self, MoveTo},
    event::{DisableMouseCapture, EnableMouseCapture, Event},
    style::{Colors, Print, SetColors},
    terminal::{
        Clear, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
    },
};
use ratatui::{
    Terminal,
    backend::{Backend, CrosstermBackend, TestBackend},
    buffer::Cell,
    layout::Size,
    prelude::{Position, backend::WindowSize},
    style::{Color, Style},
};
use std::time::Duration;
use std::{
    borrow::Cow,
    io::{self, Stdout, stdout},
};

pub type Term = Terminal<TermBackend>;

pub fn backend() -> TermBackend {
    TermBackend::Crossterm(CrosstermBackend::new(stdout()))
}

pub enum TermBackend {
    Crossterm(CrosstermBackend<Stdout>),
    #[allow(dead_code)]
    Test {
        backend: TestBackend,
        events: Vec<Event>,
    },
}

impl TermBackend {
    pub(crate) fn queue_move_cursor(&mut self, x: u16, y: u16) -> Res<()> {
        match self {
            TermBackend::Crossterm(t) => crossterm::queue!(t, MoveTo(x, y)).map_err(Error::Term),
            TermBackend::Test { backend, events } => todo!(),
        }
    }

    pub(crate) fn queue_clear(&mut self) -> Res<()> {
        match self {
            TermBackend::Crossterm(t) => {
                crossterm::queue!(t, Clear(crossterm::terminal::ClearType::All))
                    .map_err(Error::Term)
            }
            TermBackend::Test { backend, events } => todo!(),
        }
    }

    pub fn queue_print<'a>(&mut self, (text, style): &(Cow<'a, str>, Style)) -> Res<()> {
        match self {
            TermBackend::Crossterm(t) => {
                let fg = style.fg.unwrap_or(Color::Reset);
                let bg = style.bg.unwrap_or(Color::Reset);

                crossterm::queue!(t, SetColors(Colors::new(fg.into(), bg.into())))
                    .map_err(Error::Term)?;

                crossterm::queue!(t, Print(text)).map_err(Error::Term)?;

                Ok(())
            }
            TermBackend::Test { backend, events } => todo!(),
        }
    }
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
    pub fn setup_term(&mut self, config: &Config) -> io::Result<()> {
        match self {
            TermBackend::Crossterm(crossterm_backend) => {
                enable_raw_mode()?;
                crossterm_backend.queue(EnterAlternateScreen)?;
                crossterm_backend.queue(cursor::Hide)?;
                if config.general.mouse_support {
                    crossterm_backend.queue(EnableMouseCapture)?;
                }
            }
            TermBackend::Test { .. } => {}
        }

        self.flush()
    }

    pub fn reset_term(&mut self, config: &Config) -> io::Result<()> {
        match self {
            TermBackend::Crossterm(crossterm_backend) => {
                if config.general.mouse_support {
                    crossterm_backend.queue(DisableMouseCapture)?;
                }
                crossterm_backend.queue(cursor::Show)?;
                crossterm_backend.queue(LeaveAlternateScreen)?;
                disable_raw_mode()?;
            }
            TermBackend::Test { .. } => {}
        }

        self.flush()
    }

    pub(crate) fn reset_term_stay_on_alt_screeen(
        &mut self,
        config: &Config,
    ) -> Result<(), io::Error> {
        match self {
            TermBackend::Crossterm(crossterm_backend) => {
                if config.general.mouse_support {
                    crossterm_backend.queue(DisableMouseCapture)?;
                }
                crossterm_backend.queue(cursor::Show)?;
                disable_raw_mode()?;
            }
            TermBackend::Test { .. } => {}
        }

        self.flush()
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
