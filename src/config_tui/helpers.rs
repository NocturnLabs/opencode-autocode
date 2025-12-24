//! Shared UI helpers for config TUI sections

use console::style;

/// Display the config TUI header banner
pub fn display_header() {
    println!(
        "\n{}",
        style("═══════════════════════════════════════════════════")
            .cyan()
            .bold()
    );
    println!(
        "{}",
        style("  OpenCode Autocode - Configuration").cyan().bold()
    );
    println!(
        "{}\n",
        style("═══════════════════════════════════════════════════")
            .cyan()
            .bold()
    );
}

/// Display a section header with title and description
pub fn display_section(title: &str, description: &str) {
    println!("\n{}", style("═".repeat(55)).dim());
    println!("  {}", style(title).cyan().bold());
    println!("  {}", style(description).dim());
    println!("{}\n", style("═".repeat(55)).dim());
}

/// Display a description hint before an input prompt
pub fn display_hint(description: &str) {
    println!("  {} {}", style("ℹ").blue(), style(description).dim());
}
