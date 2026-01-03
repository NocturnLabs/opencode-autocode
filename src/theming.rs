//! Shared theming and styling for TUI components
//!
//! Provides consistent colors, symbols, and box drawing characters
//! across all interactive terminal displays.

#![allow(dead_code)]

use console::{style, StyledObject};

/// RGB color palette for the TUI theme
pub mod colors {
    use console::Color;

    /// Primary accent color - soft blue
    pub const PRIMARY: Color = Color::Color256(117); // #87d7ff

    /// Success/completion - soft green
    pub const SUCCESS: Color = Color::Color256(114); // #87d787

    /// Warning/caution - amber
    pub const WARNING: Color = Color::Color256(214); // #ffaf00

    /// Error/failure - soft red
    pub const ERROR: Color = Color::Color256(203); // #ff5f5f

    /// Muted/secondary text
    pub const MUTED: Color = Color::Color256(245); // #8a8a8a

    /// Highlight - bright cyan
    pub const HIGHLIGHT: Color = Color::Color256(87); // #5fffff
}

/// Unicode symbols for status indicators
pub mod symbols {
    pub const SUCCESS: &str = "✔";
    pub const FAILURE: &str = "✗";
    pub const PENDING: &str = "○";
    pub const RUNNING: &str = "●";
    pub const ARROW: &str = "→";
    pub const BULLET: &str = "•";
    pub const INFO: &str = "ℹ";
    pub const WARNING: &str = "⚠";
    pub const SPARKLE: &str = "✨";
}

/// Box drawing characters for borders
pub mod boxes {
    // Rounded corners
    pub const TOP_LEFT: &str = "╭";
    pub const TOP_RIGHT: &str = "╮";
    pub const BOTTOM_LEFT: &str = "╰";
    pub const BOTTOM_RIGHT: &str = "╯";

    // Lines
    pub const HORIZONTAL: &str = "─";
    pub const VERTICAL: &str = "│";
    pub const HORIZONTAL_BOLD: &str = "━";

    /// Create a horizontal line of specified width
    pub fn line(width: usize) -> String {
        HORIZONTAL.repeat(width)
    }

    /// Create a bold horizontal line of specified width
    pub fn bold_line(width: usize) -> String {
        HORIZONTAL_BOLD.repeat(width)
    }

    /// Create a boxed header with rounded corners
    pub fn header(title: &str, width: usize) -> String {
        let inner_width = width.saturating_sub(2);
        let padding = inner_width.saturating_sub(title.len());
        let left_pad = padding / 2;
        let right_pad = padding - left_pad;

        format!(
            "{}{}{}\n{}{}{}{}{}\n{}{}{}",
            TOP_LEFT,
            line(inner_width),
            TOP_RIGHT,
            VERTICAL,
            " ".repeat(left_pad),
            title,
            " ".repeat(right_pad),
            VERTICAL,
            BOTTOM_LEFT,
            line(inner_width),
            BOTTOM_RIGHT
        )
    }
}

/// Styled output helpers
pub fn success<D: std::fmt::Display>(text: D) -> StyledObject<D> {
    style(text).fg(colors::SUCCESS)
}

pub fn error<D: std::fmt::Display>(text: D) -> StyledObject<D> {
    style(text).fg(colors::ERROR)
}

pub fn warning<D: std::fmt::Display>(text: D) -> StyledObject<D> {
    style(text).fg(colors::WARNING)
}

pub fn primary<D: std::fmt::Display>(text: D) -> StyledObject<D> {
    style(text).fg(colors::PRIMARY)
}

pub fn muted<D: std::fmt::Display>(text: D) -> StyledObject<D> {
    style(text).fg(colors::MUTED)
}

pub fn highlight<D: std::fmt::Display>(text: D) -> StyledObject<D> {
    style(text).fg(colors::HIGHLIGHT).bold()
}
