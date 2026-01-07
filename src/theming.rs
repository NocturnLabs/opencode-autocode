//! Shared theming and styling for TUI components
#![allow(dead_code)]
//!
//! Provides consistent colors, symbols, and box drawing characters
//! across all interactive terminal displays.

use std::fmt;

/// ANSI 256-color codes for terminal styling
pub mod colors {
    pub const PRIMARY: u8 = 117; // #87d7ff - soft blue
    pub const SECONDARY: u8 = 153; // #afd7ff - lighter blue
    pub const SUCCESS: u8 = 114; // #87d787 - soft green
    pub const WARNING: u8 = 215; // #ffaf5f - soft orange
    pub const ERROR: u8 = 203; // #ff5f5f - soft red
    pub const MUTED: u8 = 245; // #8a8a8a - gray
    pub const HIGHLIGHT: u8 = 87; // #5fffff - bright cyan
    pub const ACCENT: u8 = 177; // #d787ff - soft purple
}

/// Unicode symbols for status indicators
pub mod symbols {
    pub const SUCCESS: &str = "✔";
    pub const ERROR: &str = "✖";
    pub const WARNING: &str = "⚠";
    pub const PENDING: &str = "○";
    pub const RUNNING: &str = "●";
    pub const ARROW: &str = "→";
    pub const CHEVRON: &str = "❯";
    pub const BULLET: &str = "•";
    pub const INFO: &str = "ℹ";
    pub const SPARKLE: &str = "✨";
    pub const LOCK: &str = "🔒";
    pub const CLOCK: &str = "🕒";
}

/// Box drawing characters for borders
pub mod boxes {
    pub const TOP_LEFT: &str = "╭";
    pub const TOP_RIGHT: &str = "╮";
    pub const BOTTOM_LEFT: &str = "╰";
    pub const BOTTOM_RIGHT: &str = "╯";
    pub const HORIZONTAL: &str = "─";
    pub const VERTICAL: &str = "│";

    /// Create a horizontal line of specified width
    pub fn line(width: usize) -> String {
        HORIZONTAL.repeat(width)
    }
}

/// A styled string that applies ANSI formatting when displayed
pub struct StyledString {
    text: String,
    foreground: Option<u8>,
    bold: bool,
}

impl StyledString {
    pub fn new(text: impl ToString) -> Self {
        Self {
            text: text.to_string(),
            foreground: None,
            bold: false,
        }
    }

    pub fn fg(mut self, color: u8) -> Self {
        self.foreground = Some(color);
        self
    }

    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }
}

impl fmt::Display for StyledString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut codes = Vec::new();

        if self.bold {
            codes.push("1".to_string());
        }

        if let Some(color) = self.foreground {
            codes.push(format!("38;5;{}", color));
        }

        if codes.is_empty() {
            write!(f, "{}", self.text)
        } else {
            write!(f, "\x1b[{}m{}\x1b[0m", codes.join(";"), self.text)
        }
    }
}

/// Create a styled string with the given color
fn style<D: ToString>(text: D) -> StyledString {
    StyledString::new(text)
}

/// Styled output helpers
pub fn success<D: ToString>(text: D) -> StyledString {
    style(text).fg(colors::SUCCESS)
}

pub fn primary<D: ToString>(text: D) -> StyledString {
    style(text).fg(colors::PRIMARY)
}

pub fn muted<D: ToString>(text: D) -> StyledString {
    style(text).fg(colors::MUTED)
}

pub fn highlight<D: ToString>(text: D) -> StyledString {
    style(text).fg(colors::HIGHLIGHT).bold()
}

pub fn warning<D: ToString>(text: D) -> StyledString {
    style(text).fg(colors::WARNING)
}

pub fn error<D: ToString>(text: D) -> StyledString {
    style(text).fg(colors::ERROR)
}

pub fn accent<D: ToString>(text: D) -> StyledString {
    style(text).fg(colors::ACCENT)
}

/// Create a text-based progress bar
pub fn progress_bar(current: usize, total: usize, width: usize) -> String {
    let percentage = if total > 0 {
        current as f64 / total as f64
    } else {
        0.0
    };
    let filled_width = (percentage * width as f64).round() as usize;
    let empty_width = width.saturating_sub(filled_width);

    let filled = "█".repeat(filled_width);
    let empty = "░".repeat(empty_width);

    format!("{}{}", success(filled), muted(empty))
}
