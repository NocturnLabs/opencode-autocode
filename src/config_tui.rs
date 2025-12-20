//! Configuration TUI for editing autocode.toml

use anyhow::{Context, Result};
use console::style;
use dialoguer::{Input, Select};
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::config::Config;

/// Run the configuration TUI
pub fn run_config_tui() -> Result<()> {
    print_header();

    // Load existing config or defaults
    let config_path = Path::new("autocode.toml");
    let mut config = if config_path.exists() {
        Config::load_from_file(config_path)?
    } else {
        Config::default()
    };

    // Fetch available models
    let models = fetch_available_models()?;
    println!(
        "{}",
        style(format!("Found {} available models", models.len())).dim()
    );

    // Model selection
    config.models.autonomous = select_model(
        "Autonomous model (for vibe loop)",
        &models,
        &config.models.autonomous,
    )?;

    config.models.default = select_model(
        "Default model (for spec generation)",
        &models,
        &config.models.default,
    )?;

    // Loop settings
    println!("\n{}", style("─── Loop Settings ───").yellow().bold());

    config.autonomous.max_iterations = Input::new()
        .with_prompt("Max iterations (0 = unlimited)")
        .default(config.autonomous.max_iterations)
        .interact()?;

    config.autonomous.delay_between_sessions = Input::new()
        .with_prompt("Delay between sessions (seconds)")
        .default(config.autonomous.delay_between_sessions)
        .interact()?;

    // Alternative approaches settings
    println!(
        "\n{}",
        style("─── Stuck Recovery Settings ───").yellow().bold()
    );

    config.alternative_approaches.retry_threshold = Input::new()
        .with_prompt("Retry threshold before generating alternatives")
        .default(config.alternative_approaches.retry_threshold)
        .interact()?;

    config.alternative_approaches.num_approaches = Input::new()
        .with_prompt("Number of alternative approaches to generate")
        .default(config.alternative_approaches.num_approaches)
        .interact()?;

    // Save config
    save_config(&config, config_path)?;

    println!(
        "\n{}",
        style("✅ Configuration saved to autocode.toml").green().bold()
    );

    Ok(())
}

fn print_header() {
    println!(
        "\n{}",
        style("═══════════════════════════════════════════════════").cyan()
    );
    println!(
        "{}",
        style("  OpenCode Autocode - Configuration").cyan().bold()
    );
    println!(
        "{}\n",
        style("═══════════════════════════════════════════════════").cyan()
    );
}

/// Fetch available models from opencode
fn fetch_available_models() -> Result<Vec<String>> {
    println!("{}", style("Fetching available models...").dim());

    let output = Command::new("opencode")
        .arg("models")
        .output()
        .context("Failed to run 'opencode models'. Is OpenCode installed?")?;

    if !output.status.success() {
        println!(
            "{}",
            style("Warning: Could not fetch models, using defaults").yellow()
        );
        return Ok(vec![
            "opencode/grok-code".to_string(),
            "opencode/claude-sonnet-4".to_string(),
            "opencode/gemini-3-pro".to_string(),
        ]);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let models: Vec<String> = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|s| s.trim().to_string())
        .collect();

    if models.is_empty() {
        Ok(vec!["opencode/grok-code".to_string()])
    } else {
        Ok(models)
    }
}

/// Select a model from available options or enter custom
fn select_model(prompt: &str, models: &[String], current: &str) -> Result<String> {
    println!("\n{}", style(format!("─── {} ───", prompt)).yellow().bold());

    // Find current model index or add it
    let mut options: Vec<String> = models.to_vec();
    let current_idx = options.iter().position(|m| m == current);

    // Add current if not in list
    if current_idx.is_none() && !current.is_empty() {
        options.insert(0, current.to_string());
    }

    // Add custom option
    options.push("(Enter custom model)".to_string());

    let default_idx = options
        .iter()
        .position(|m| m == current)
        .unwrap_or(0);

    let idx = Select::new()
        .with_prompt("Select model")
        .items(&options)
        .default(default_idx)
        .interact()?;

    if idx == options.len() - 1 {
        // Custom model
        let custom: String = Input::new()
            .with_prompt("Enter model (provider/model)")
            .default(current.to_string())
            .interact_text()?;
        Ok(custom)
    } else {
        Ok(options[idx].clone())
    }
}

/// Save config to TOML file
fn save_config(config: &Config, path: &Path) -> Result<()> {
    // Generate TOML content matching the actual Config struct fields
    let content = format!(
        r#"# OpenCode Autocode Configuration
# Generated by opencode-autocode --config

[models]
default = "{}"
autonomous = "{}"
reasoning = "{}"
enhancement = "{}"

[autonomous]
max_iterations = {}
delay_between_sessions = {}
log_level = "{}"

[alternative_approaches]
enabled = {}
num_approaches = {}
retry_threshold = {}
cache_results = {}
cache_dir = "{}"

[agent]
max_retry_attempts = {}
max_research_attempts = {}
verification_sample_size = {}
single_feature_focus = {}

[paths]
feature_list_file = "{}"
progress_file = "{}"
log_dir = "{}"
"#,
        config.models.default,
        config.models.autonomous,
        config.models.reasoning,
        config.models.enhancement,
        config.autonomous.max_iterations,
        config.autonomous.delay_between_sessions,
        config.autonomous.log_level,
        config.alternative_approaches.enabled,
        config.alternative_approaches.num_approaches,
        config.alternative_approaches.retry_threshold,
        config.alternative_approaches.cache_results,
        config.alternative_approaches.cache_dir,
        config.agent.max_retry_attempts,
        config.agent.max_research_attempts,
        config.agent.verification_sample_size,
        config.agent.single_feature_focus,
        config.paths.feature_list_file,
        config.paths.progress_file,
        config.paths.log_dir,
    );

    fs::write(path, content).context("Failed to write autocode.toml")?;
    Ok(())
}
