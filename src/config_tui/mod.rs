//! Configuration TUI for editing autocode.toml
//!
//! This module provides an interactive terminal interface for configuring
//! all aspects of the opencode-autocode tool.

mod helpers;
mod models;
mod save;
mod sections;

use anyhow::Result;
use console::style;
use std::path::Path;

use crate::config::Config;

// Re-export for external use
pub use models::fetch_available_models;

/// Run the configuration TUI
pub fn run_config_tui() -> Result<()> {
    helpers::display_header();

    // Load existing config or defaults
    let config_path = Path::new("autocode.toml");
    let mut config = if config_path.exists() {
        Config::load_from_file(config_path)?
    } else {
        Config::default()
    };

    // Fetch available models
    let available_models = models::fetch_available_models()?;
    println!(
        "{}\n",
        style(format!("Found {} available models", available_models.len())).dim()
    );

    // Configure each section
    sections::configure_models(&mut config, &available_models)?;
    sections::configure_autonomous(&mut config)?;
    sections::configure_agent(&mut config)?;
    sections::configure_stuck_recovery(&mut config)?;
    sections::configure_mcp(&mut config)?;
    sections::configure_conductor(&mut config)?;
    sections::configure_generation(&mut config)?;
    sections::configure_security(&mut config)?;
    sections::configure_notifications(&mut config)?;
    sections::configure_ui(&mut config)?;

    // Save configuration files
    save::save_autocode_toml(&config, config_path)?;
    save::save_opencode_json(&config, Path::new("opencode.json"))?;

    println!(
        "\n{}",
        style("✅ Configuration saved to autocode.toml and opencode.json")
            .green()
            .bold()
    );

    display_next_steps();
    Ok(())
}

fn display_next_steps() {
    println!("\n{}", style("─── Next Steps ───").cyan().bold());
    println!(
        "  {} Run {} to start the autonomous coding loop",
        style("→").cyan(),
        style("opencode-autocode vibe").green().bold()
    );
    println!(
        "  {} Run {} to modify settings again",
        style("→").cyan(),
        style("opencode-autocode --config").dim()
    );
    println!(
        "  {} Edit {} directly for advanced options",
        style("→").cyan(),
        style("autocode.toml").dim()
    );
    println!();
}
