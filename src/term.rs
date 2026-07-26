use crate::{Res, config::Config, error::Error};
use crossterm::{
    QueueableCommand,
    cursor::{self, MoveTo},
    event::{DisableMouseCapture, EnableMouseCapture, Event},
    style::{Attribute, Colors, Print, SetAttribute, SetColors},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    backend::{Backend, CrosstermBackend, TestBackend},
    buffer::Cell,
    prelude::Position,
    style::{Color, Style},
};
use std::io::{self, Stdout, stdout};
use std::time::Duration;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

pub type Term = TermBackend;

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
            TermBackend::Test { backend, .. } => backend
                .set_cursor_position(Position::new(x, y))
                .map_err(Error::Term),
        }
    }

    pub fn queue_print(&mut self, text: &str, style: &Style) -> Res<()> {
        match self {
            TermBackend::Crossterm(t) => {
                print_crossterm_span(text, style, t).map_err(Error::Term)?;

                Ok(())
            }
            TermBackend::Test { backend, .. } => {
                print_test_span(text, style, backend).map_err(Error::Term)
            }
        }
    }
}

const ATTRS: &[(ratatui::style::Modifier, Attribute)] = &[
    (ratatui::style::Modifier::BOLD, Attribute::Bold),
    (ratatui::style::Modifier::DIM, Attribute::Dim),
    (ratatui::style::Modifier::ITALIC, Attribute::Italic),
    (ratatui::style::Modifier::UNDERLINED, Attribute::Underlined),
    (ratatui::style::Modifier::SLOW_BLINK, Attribute::SlowBlink),
    (ratatui::style::Modifier::RAPID_BLINK, Attribute::RapidBlink),
    (ratatui::style::Modifier::REVERSED, Attribute::Reverse),
    (ratatui::style::Modifier::HIDDEN, Attribute::Hidden),
    (ratatui::style::Modifier::CROSSED_OUT, Attribute::CrossedOut),
];

fn print_crossterm_span(
    text: &str,
    style: &Style,
    t: &mut CrosstermBackend<Stdout>,
) -> Result<(), io::Error> {
    let fg = style.fg.unwrap_or(Color::Reset);
    let bg = style.bg.unwrap_or(Color::Reset);

    crossterm::queue!(t, SetAttribute(Attribute::Reset))?;

    for (modifier, attribute) in ATTRS {
        if style.add_modifier.contains(*modifier) {
            crossterm::queue!(t, SetAttribute(*attribute))?;
        }
    }

    crossterm::queue!(t, SetColors(Colors::new(fg.into(), bg.into())))?;
    crossterm::queue!(t, Print(text))?;
    Ok(())
}

fn print_test_span(text: &str, style: &Style, backend: &mut TestBackend) -> io::Result<()> {
    let Position { x, y } = backend.get_cursor_position()?;
    let width = backend.size()?.width;

    let mut cells = Vec::new();
    let mut cx = x;
    for grapheme in text.graphemes(true) {
        let grapheme_width = grapheme.width() as u16;
        if grapheme_width == 0 || cx >= width {
            continue;
        }

        let mut cell = Cell::default();
        cell.set_symbol(grapheme).set_style(*style);
        cells.push((cx, y, cell));
        cx += 1;

        for _ in 1..grapheme_width {
            if cx >= width {
                break;
            }
            cells.push((cx, y, Cell::default()));
            cx += 1;
        }
    }

    backend.draw(cells.iter().map(|(x, y, cell)| (*x, *y, cell)))?;
    backend.set_cursor_position(Position::new(cx, y))
}

impl TermBackend {
    pub(crate) fn size(&self) -> io::Result<(u16, u16)> {
        match self {
            TermBackend::Crossterm(_) => crossterm::terminal::size(),
            TermBackend::Test { backend, .. } => {
                let size = backend.size()?;
                Ok((size.width, size.height))
            }
        }
    }

    pub(crate) fn clear(&mut self) -> io::Result<()> {
        match self {
            TermBackend::Crossterm(t) => t.clear(),
            TermBackend::Test { backend, .. } => backend.clear(),
        }
    }

    pub(crate) fn flush(&mut self) -> io::Result<()> {
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
