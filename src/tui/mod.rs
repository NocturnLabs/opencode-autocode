//! Interactive TUI for building app specs
//!
//! Provides multiple modes for creating project specifications:
//! - Generated: AI creates spec from idea
//! - Manual: Step-by-step form
//! - From file: Use existing spec
//! - Default: Built-in template

mod actions;
mod generated;
mod manual;
mod validation;

use anyhow::Result;
use console::style;
use dialoguer::{Confirm, Input, Select};
use std::path::Path;

use crate::scaffold::{scaffold_custom, scaffold_default};

/// Interactive mode options
enum InteractiveMode {
    Generated,
    Manual,
    FromSpecFile,
    Default,
}

/// Run the interactive TUI to build an app spec
pub fn run_interactive(output_dir: &Path, use_subagents: bool) -> Result<()> {
    display_header();

    // Load config BEFORE spec generation
    let config = load_or_configure_config(output_dir)?;

    match select_mode()? {
        InteractiveMode::Generated => {
            generated::run_generated_mode(output_dir, &config, use_subagents)
        }
        InteractiveMode::Manual => manual::run_manual_mode(output_dir),
        InteractiveMode::FromSpecFile => run_from_spec_file_mode(output_dir),
        InteractiveMode::Default => run_default_mode(output_dir),
    }
}

/// Load existing config or prompt user to configure/use defaults
fn load_or_configure_config(output_dir: &Path) -> Result<crate::config::Config> {
    use crate::config::Config;
    use crate::config_tui::run_config_tui;

    let config_path = output_dir.join(".autocode/config.toml");

    if config_path.exists() {
        // Existing config found
        if Confirm::new()
            .with_prompt("Found existing config. Reconfigure?")
            .default(false)
            .interact()?
        {
            run_config_tui()?;
        }
        Ok(Config::load(Some(output_dir)).unwrap_or_default())
    } else {
        // No config: quick start or configure
        let choice = Select::new()
            .with_prompt("Setup mode")
            .items([
                "⚡ Quick start (use defaults)",
                "⚙️  Configure settings first",
            ])
            .default(0)
            .interact()?;

        if choice == 1 {
            run_config_tui()?;
            Ok(Config::load(Some(output_dir)).unwrap_or_default())
        } else {
            Ok(Config::default())
        }
    }
}

fn display_header() {
    println!(
        "\n{}",
        style("═══════════════════════════════════════════════════").cyan()
    );
    println!(
        "{}",
        style("  OpenCode Autonomous Plugin - Interactive Setup")
            .cyan()
            .bold()
    );
    println!(
        "{}\n",
        style("═══════════════════════════════════════════════════").cyan()
    );
}

fn select_mode() -> Result<InteractiveMode> {
    let mode_idx = Select::new()
        .with_prompt("How would you like to create your project spec?")
        .items([
            "🤖 Generated - AI researches and creates full spec",
            "📝 Manual - Fill out project details step by step",
            "📁 From file - Use an existing app_spec.md",
            "⚡ Default - Use built-in specification",
        ])
        .default(0)
        .interact()?;

    Ok(match mode_idx {
        0 => InteractiveMode::Generated,
        1 => InteractiveMode::Manual,
        2 => InteractiveMode::FromSpecFile,
        _ => InteractiveMode::Default,
    })
}

fn run_from_spec_file_mode(output_dir: &Path) -> Result<()> {
    println!("\n{}", style("─── Spec File Mode ───").yellow().bold());

    let spec_path: String = Input::new()
        .with_prompt("Path to spec file")
        .default("app_spec.md".to_string())
        .interact_text()?;

    let spec_path = std::path::PathBuf::from(&spec_path);
    if !spec_path.exists() {
        println!("{} Spec file not found.", style("Error:").red().bold());
        return Ok(());
    }

    scaffold_custom(output_dir, &spec_path)?;
    println!(
        "\n{}",
        style("✅ Project scaffolded from spec file!")
            .green()
            .bold()
    );
    Ok(())
}

fn run_default_mode(output_dir: &Path) -> Result<()> {
    println!("\n{}", style("─── Default Mode ───").yellow().bold());
    println!(
        "{}",
        style("Using the built-in default specification.").dim()
    );

    if Confirm::new()
        .with_prompt("Scaffold project with default spec?")
        .default(true)
        .interact()?
    {
        scaffold_default(output_dir)?;
        println!(
            "\n{}",
            style("✅ Project scaffolded with default spec!")
                .green()
                .bold()
        );
    } else {
        println!("{}", style("Cancelled.").red());
    }
    Ok(())
}
