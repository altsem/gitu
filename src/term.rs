use crate::style::{Color, Modifier, Style};
use crate::{Res, config::Config, error::Error};
use crossterm::{
    QueueableCommand,
    cursor::{self, MoveTo},
    event::{DisableMouseCapture, EnableMouseCapture, Event},
    style::{Attribute, Colors, Print, SetAttribute, SetColors},
    terminal::{
        Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode,
    },
};
use std::io::{self, Stdout, Write, stdout};
use std::time::Duration;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

pub type Term = TermBackend;

pub fn backend() -> TermBackend {
    TermBackend::Crossterm(stdout())
}

pub enum TermBackend {
    Crossterm(Stdout),
    #[allow(dead_code)]
    Test {
        buffer: TestBuffer,
        events: Vec<Event>,
    },
}

#[derive(Clone, PartialEq)]
pub struct TestCell {
    pub symbol: String,
    pub fg: Color,
    pub bg: Color,
    pub modifier: Modifier,
}

impl Default for TestCell {
    fn default() -> Self {
        TestCell {
            symbol: " ".to_string(),
            fg: Color::Reset,
            bg: Color::Reset,
            modifier: Modifier::empty(),
        }
    }
}

impl TestCell {
    fn set(&mut self, symbol: &str, style: &Style) {
        self.symbol.clear();
        self.symbol.push_str(symbol);

        if let Some(fg) = style.fg {
            self.fg = fg;
        }
        if let Some(bg) = style.bg {
            self.bg = bg;
        }
        self.modifier.insert(style.add_modifier);
        self.modifier.remove(style.sub_modifier);
    }
}

pub struct TestBuffer {
    pub cells: Vec<TestCell>,
    pub width: u16,
    pub height: u16,
    cursor: (u16, u16),
}

impl TestBuffer {
    pub fn new(width: u16, height: u16) -> Self {
        TestBuffer {
            cells: vec![TestCell::default(); width as usize * height as usize],
            width,
            height,
            cursor: (0, 0),
        }
    }

    fn clear(&mut self) {
        self.cells.fill(TestCell::default());
    }

    fn cell_mut(&mut self, x: u16, y: u16) -> Option<&mut TestCell> {
        if x >= self.width || y >= self.height {
            return None;
        }

        self.cells
            .get_mut(y as usize * self.width as usize + x as usize)
    }

    fn print(&mut self, text: &str, style: &Style) {
        let (mut x, y) = self.cursor;

        for grapheme in text.graphemes(true) {
            let grapheme_width = grapheme.width() as u16;
            if grapheme_width == 0 || x >= self.width {
                continue;
            }

            if let Some(cell) = self.cell_mut(x, y) {
                *cell = TestCell::default();
                cell.set(grapheme, style);
            }
            x += 1;

            for _ in 1..grapheme_width {
                if x >= self.width {
                    break;
                }

                if let Some(cell) = self.cell_mut(x, y) {
                    *cell = TestCell::default();
                }
                x += 1;
            }
        }

        self.cursor = (x, y);
    }
}

impl TermBackend {
    pub(crate) fn queue_move_cursor(&mut self, x: u16, y: u16) -> Res<()> {
        match self {
            TermBackend::Crossterm(t) => crossterm::queue!(t, MoveTo(x, y)).map_err(Error::Term),
            TermBackend::Test { buffer, .. } => {
                buffer.cursor = (x, y);
                Ok(())
            }
        }
    }

    pub fn queue_print(&mut self, text: &str, style: &Style) -> Res<()> {
        match self {
            TermBackend::Crossterm(t) => {
                print_crossterm_span(text, style, t).map_err(Error::Term)?;

                Ok(())
            }
            TermBackend::Test { buffer, .. } => {
                buffer.print(text, style);
                Ok(())
            }
        }
    }
}

const ATTRS: &[(Modifier, Attribute)] = &[
    (Modifier::BOLD, Attribute::Bold),
    (Modifier::DIM, Attribute::Dim),
    (Modifier::ITALIC, Attribute::Italic),
    (Modifier::UNDERLINED, Attribute::Underlined),
    (Modifier::SLOW_BLINK, Attribute::SlowBlink),
    (Modifier::RAPID_BLINK, Attribute::RapidBlink),
    (Modifier::REVERSED, Attribute::Reverse),
    (Modifier::HIDDEN, Attribute::Hidden),
    (Modifier::CROSSED_OUT, Attribute::CrossedOut),
];

fn print_crossterm_span(text: &str, style: &Style, t: &mut Stdout) -> Result<(), io::Error> {
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

impl TermBackend {
    pub(crate) fn size(&self) -> io::Result<(u16, u16)> {
        match self {
            TermBackend::Crossterm(_) => crossterm::terminal::size(),
            TermBackend::Test { buffer, .. } => Ok((buffer.width, buffer.height)),
        }
    }

    pub(crate) fn clear(&mut self) -> io::Result<()> {
        match self {
            TermBackend::Crossterm(t) => crossterm::queue!(t, Clear(ClearType::All)),
            TermBackend::Test { buffer, .. } => {
                buffer.clear();
                Ok(())
            }
        }
    }

    pub(crate) fn flush(&mut self) -> io::Result<()> {
        match self {
            TermBackend::Crossterm(t) => t.flush(),
            TermBackend::Test { .. } => Ok(()),
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
