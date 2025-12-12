use base64::{engine::general_purpose::STANDARD, Engine};
use std::io::{self, Write};

/// Trait for clipboard implementations
trait ClipboardBackend {
    fn set_text(&mut self, text: &str) -> Result<(), ClipboardError>;
}

/// Arboard-based clipboard implementation (system clipboard)
struct ArboardClipboard {
    clipboard: arboard::Clipboard,
}

impl ArboardClipboard {
    fn new() -> Result<Self, arboard::Error> {
        Ok(Self {
            clipboard: arboard::Clipboard::new()?,
        })
    }
}

impl ClipboardBackend for ArboardClipboard {
    fn set_text(&mut self, text: &str) -> Result<(), ClipboardError> {
        self.clipboard
            .set_text(text)
            .map_err(|e| ClipboardError::Arboard(e))
    }
}

/// OSC 52-based clipboard implementation (terminal escape sequences)
struct Osc52Clipboard;

impl Osc52Clipboard {
    fn new() -> Self {
        Self
    }
}

impl ClipboardBackend for Osc52Clipboard {
    fn set_text(&mut self, text: &str) -> Result<(), ClipboardError> {
        let encoded = STANDARD.encode(text);
        let osc52 = format!("\x1b]52;c;{}\x07", encoded);

        // Write directly to stderr to avoid interfering with UI
        io::stderr().write_all(osc52.as_bytes())?;
        io::stderr().flush()?;

        Ok(())
    }
}

/// Main clipboard wrapper that manages different backend implementations
pub(crate) struct Clipboard {
    backend: Box<dyn ClipboardBackend>,
}

impl Clipboard {
    /// Creates a new clipboard instance with the preferred backend.
    /// If use_osc52 is true, uses OSC 52 with arboard as fallback.
    /// Otherwise, uses only arboard.
    pub fn new(use_osc52: bool) -> Option<Self> {
        if use_osc52 {
            // Prefer OSC 52, fallback to arboard
            Some(Self {
                backend: Box::new(Osc52Clipboard::new()),
            })
        } else {
            // Try arboard only
            ArboardClipboard::new()
                .inspect_err(|e| log::warn!("Couldn't initialize arboard clipboard: {e}"))
                .ok()
                .map(|cb| Self {
                    backend: Box::new(cb),
                })
        }
    }

    /// Sets text to clipboard using the configured backend.
    pub fn set_text(&mut self, text: String) -> Result<(), ClipboardError> {
        self.backend.set_text(&text)
    }
}

#[derive(Debug)]
pub enum ClipboardError {
    Io(io::Error),
    Arboard(arboard::Error),
}

impl From<io::Error> for ClipboardError {
    fn from(err: io::Error) -> Self {
        ClipboardError::Io(err)
    }
}

impl std::fmt::Display for ClipboardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClipboardError::Io(err) => write!(f, "IO error: {}", err),
            ClipboardError::Arboard(err) => write!(f, "Clipboard error: {}", err),
        }
    }
}

impl std::error::Error for ClipboardError {}
