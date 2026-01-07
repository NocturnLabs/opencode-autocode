//! Configuration TUI for editing forger.toml
//!
//! This module provides an interactive terminal interface for configuring
//! all aspects of the opencode-forger tool.

mod models;
mod save;

use anyhow::Result;
use std::path::Path;

use crate::config::Config;

/// Run the configuration TUI
pub fn run_config_tui(dir: Option<&Path>) -> Result<Config> {
    // Load existing config or defaults
    let mut config = Config::load(dir).unwrap_or_default();
    let config_path = match dir {
        Some(d) => d.join(".forger/config.toml"),
        None => std::path::PathBuf::from(".forger/config.toml"),
    };

    // Fetch available models
    let available_models = models::fetch_available_models().unwrap_or_default();

    // Use fullscreen TUI for configuration
    crate::tui::run_fullscreen_config_editor(&mut config, available_models)?;

    // Save configuration files
    save::save_forger_toml(&config, &config_path)?;
    let opencode_json_path = match dir {
        Some(d) => d.join("opencode.json"),
        None => std::path::PathBuf::from("opencode.json"),
    };
    save::save_opencode_json(&config, &opencode_json_path)?;

    println!("\n✅ Configuration saved.");
    display_next_steps();
    Ok(config)
}

fn display_next_steps() {
    println!("\n─── Next Steps ───");
    println!("  → Run opencode-forger vibe to start the autonomous coding loop");
    println!("  → Run opencode-forger --config to modify settings again");
    println!("  → Edit .forger/config.toml directly for advanced options");
    println!();
}
