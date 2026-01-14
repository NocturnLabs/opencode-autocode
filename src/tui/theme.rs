//! Shared theme tokens for the iocraft-based TUI.

use iocraft::prelude::Color;

/// Centralized color palette for interactive terminal components.
pub struct TuiTheme;

impl TuiTheme {
    /// Primary text color for headers and key labels.
    pub const PRIMARY: Color = Color::White;
    /// Accent color for selections and active indicators.
    pub const ACCENT: Color = Color::Cyan;
    /// Muted color for secondary hints and helper text.
    pub const MUTED: Color = Color::DarkGrey;
    /// Border color for panels and separators.
    pub const BORDER: Color = Color::DarkGrey;
    /// Success color for positive feedback.
    pub const SUCCESS: Color = Color::Green;
    /// Error color for failure states.
    pub const ERROR: Color = Color::Red;
}
