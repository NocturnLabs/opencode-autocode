//! Shared UI helpers for config TUI sections

use crate::theming::{boxes, highlight, muted, primary, symbols};

/// Display the config TUI header banner
pub fn display_header() {
    let width = 55;
    println!();
    println!(
        "{}{}{}",
        primary(boxes::TOP_LEFT),
        primary(boxes::line(width - 2)),
        primary(boxes::TOP_RIGHT)
    );
    println!(
        "{} {} OpenCode Autocode - Configuration {}{}",
        primary(boxes::VERTICAL),
        symbols::SPARKLE,
        " ".repeat(width - 42),
        primary(boxes::VERTICAL)
    );
    println!(
        "{}{}{}",
        primary(boxes::BOTTOM_LEFT),
        primary(boxes::line(width - 2)),
        primary(boxes::BOTTOM_RIGHT)
    );
    println!();
}

/// Display a section header with title and description
pub fn display_section(title: &str, description: &str) {
    println!();
    println!(
        "{}{}{}",
        muted(boxes::TOP_LEFT),
        muted(boxes::line(53)),
        muted(boxes::TOP_RIGHT)
    );
    println!("  {}", highlight(title));
    println!("  {}", muted(description));
    println!(
        "{}{}{}",
        muted(boxes::BOTTOM_LEFT),
        muted(boxes::line(53)),
        muted(boxes::BOTTOM_RIGHT)
    );
    println!();
}

/// Display a description hint before an input prompt
pub fn display_hint(description: &str) {
    println!("  {} {}", primary(symbols::INFO), muted(description));
}
