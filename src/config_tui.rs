//! Configuration TUI for editing autocode.toml

use anyhow::{Context, Result};
use console::style;
use dialoguer::{Confirm, FuzzySelect, Input, Select};
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
        "{}\n",
        style(format!("Found {} available models", models.len())).dim()
    );

    // ─────────────────────────────────────────────────────────────────────────
    // MODELS SECTION
    // ─────────────────────────────────────────────────────────────────────────
    print_section("Models", "Configure which AI models to use for different tasks");

    print_desc("Model used for the autonomous vibe loop coding sessions");
    config.models.autonomous = select_model("Autonomous model", &models, &config.models.autonomous)?;

    print_desc("Model used for generating initial app specifications");
    config.models.default = select_model("Default/Spec model", &models, &config.models.default)?;

    print_desc("Model used for reasoning and planning complex decisions");
    config.models.reasoning = select_model("Reasoning model", &models, &config.models.reasoning)?;

    print_desc("Model used for discovering enhancement opportunities");
    config.models.enhancement = select_model("Enhancement model", &models, &config.models.enhancement)?;

    // ─────────────────────────────────────────────────────────────────────────
    // AUTONOMOUS SECTION
    // ─────────────────────────────────────────────────────────────────────────
    print_section("Autonomous Loop", "Control how the vibe loop runs");

    print_desc("Maximum number of loop iterations (0 = run forever until all features pass)");
    config.autonomous.max_iterations = Input::new()
        .with_prompt("Max iterations")
        .default(config.autonomous.max_iterations)
        .interact()?;

    print_desc("Seconds to wait between loop iterations (prevents rate limiting)");
    config.autonomous.delay_between_sessions = Input::new()
        .with_prompt("Delay between sessions (seconds)")
        .default(config.autonomous.delay_between_sessions)
        .interact()?;

    print_desc("Maximum time in minutes for a single session (0 = no timeout)");
    config.autonomous.session_timeout_minutes = Input::new()
        .with_prompt("Session timeout (minutes)")
        .default(config.autonomous.session_timeout_minutes)
        .interact()?;

    print_desc("Automatically commit changes when a feature is completed");
    config.autonomous.auto_commit = Confirm::new()
        .with_prompt("Auto-commit on feature completion?")
        .default(config.autonomous.auto_commit)
        .interact()?;

    print_desc("Logging verbosity: DEBUG, INFO, WARN, ERROR");
    config.autonomous.log_level = select_from_list(
        "Log level",
        &["DEBUG", "INFO", "WARN", "ERROR"],
        &config.autonomous.log_level,
    )?;

    // ─────────────────────────────────────────────────────────────────────────
    // AGENT BEHAVIOR SECTION
    // ─────────────────────────────────────────────────────────────────────────
    print_section("Agent Behavior", "Fine-tune how the AI agent approaches tasks");

    print_desc("How many times to retry a failing feature before triggering research mode");
    config.agent.max_retry_attempts = Input::new()
        .with_prompt("Max retry attempts")
        .default(config.agent.max_retry_attempts)
        .interact()?;

    print_desc("How many research-based attempts before giving up on a feature");
    config.agent.max_research_attempts = Input::new()
        .with_prompt("Max research attempts")
        .default(config.agent.max_research_attempts)
        .interact()?;

    print_desc("Number of previously-passing features to re-verify each session (regression check)");
    config.agent.verification_sample_size = Input::new()
        .with_prompt("Verification sample size")
        .default(config.agent.verification_sample_size)
        .interact()?;

    print_desc("Focus on completing one feature at a time (recommended for better results)");
    config.agent.single_feature_focus = Confirm::new()
        .with_prompt("Single feature focus?")
        .default(config.agent.single_feature_focus)
        .interact()?;

    // ─────────────────────────────────────────────────────────────────────────
    // ALTERNATIVE APPROACHES (STUCK RECOVERY) SECTION
    // ─────────────────────────────────────────────────────────────────────────
    print_section(
        "Stuck Recovery",
        "Configure alternative approach generation when the agent gets stuck",
    );

    print_desc("Enable/disable generating alternative approaches when stuck on a feature");
    config.alternative_approaches.enabled = Confirm::new()
        .with_prompt("Enable alternative approaches?")
        .default(config.alternative_approaches.enabled)
        .interact()?;

    if config.alternative_approaches.enabled {
        print_desc("Number of retries before triggering alternative approach generation");
        config.alternative_approaches.retry_threshold = Input::new()
            .with_prompt("Retry threshold")
            .default(config.alternative_approaches.retry_threshold)
            .interact()?;

        print_desc("How many alternative approaches to generate (more = higher chance of finding solution)");
        config.alternative_approaches.num_approaches = Input::new()
            .with_prompt("Number of approaches")
            .default(config.alternative_approaches.num_approaches)
            .interact()?;

        print_desc("Cache generated approaches to avoid regenerating on restart");
        config.alternative_approaches.cache_results = Confirm::new()
            .with_prompt("Cache results?")
            .default(config.alternative_approaches.cache_results)
            .interact()?;
    }

    // ─────────────────────────────────────────────────────────────────────────
    // MCP TOOLS SECTION
    // ─────────────────────────────────────────────────────────────────────────
    print_section(
        "MCP Tools",
        "Configure Model Context Protocol tools for enhanced capabilities",
    );

    print_desc("Use osgrep (semantic code search) instead of grep when available");
    config.mcp.prefer_osgrep = Confirm::new()
        .with_prompt("Prefer osgrep over grep?")
        .default(config.mcp.prefer_osgrep)
        .interact()?;

    print_desc("Use sequential thinking MCP for complex decision-making");
    config.mcp.use_sequential_thinking = Confirm::new()
        .with_prompt("Use sequential thinking?")
        .default(config.mcp.use_sequential_thinking)
        .interact()?;

    print_desc("Required MCP tools (e.g., 'chrome-devtools' for web projects)");
    let required_tools_str = config.mcp.required_tools.join(", ");
    let new_required: String = Input::new()
        .with_prompt("Required tools (comma-separated)")
        .default(required_tools_str)
        .allow_empty(true)
        .interact_text()?;
    config.mcp.required_tools = parse_comma_list(&new_required);

    // ─────────────────────────────────────────────────────────────────────────
    // CONDUCTOR SECTION
    // ─────────────────────────────────────────────────────────────────────────
    print_section(
        "Conductor (Context-Driven Planning)",
        "Configure how project context and planning is managed",
    );

    print_desc("Directory for storing project context files (product.md, tech_stack.md, etc.)");
    config.conductor.context_dir = Input::new()
        .with_prompt("Context directory")
        .default(config.conductor.context_dir.clone())
        .interact_text()?;

    print_desc("Directory for track-based work units (per-feature specs and plans)");
    config.conductor.tracks_dir = Input::new()
        .with_prompt("Tracks directory")
        .default(config.conductor.tracks_dir.clone())
        .interact_text()?;

    print_desc("Automatically set up conductor files on first run");
    config.conductor.auto_setup = Confirm::new()
        .with_prompt("Auto-setup conductor files?")
        .default(config.conductor.auto_setup)
        .interact()?;

    print_desc("Planning mode: 'auto' (AI generates plans) or 'manual' (user writes plans)");
    config.conductor.planning_mode = select_from_list(
        "Planning mode",
        &["auto", "manual"],
        &config.conductor.planning_mode,
    )?;

    print_desc("Save progress checkpoint after completing N tasks (lower = more frequent saves)");
    config.conductor.checkpoint_frequency = Input::new()
        .with_prompt("Checkpoint frequency")
        .default(config.conductor.checkpoint_frequency)
        .interact()?;

    // ─────────────────────────────────────────────────────────────────────────
    // GENERATION SECTION
    // ─────────────────────────────────────────────────────────────────────────
    print_section(
        "Spec Generation",
        "Configure how app specifications are generated",
    );

    print_desc("Complexity level for generated specs: 'comprehensive' (detailed) or 'minimal' (lean)");
    config.generation.complexity = select_from_list(
        "Complexity",
        &["comprehensive", "minimal"],
        &config.generation.complexity,
    )?;

    print_desc("Include security considerations section in generated specs");
    config.generation.include_security_section = Confirm::new()
        .with_prompt("Include security section?")
        .default(config.generation.include_security_section)
        .interact()?;

    print_desc("Include testing strategy section in generated specs");
    config.generation.include_testing_strategy = Confirm::new()
        .with_prompt("Include testing strategy?")
        .default(config.generation.include_testing_strategy)
        .interact()?;

    print_desc("Include DevOps/deployment section in generated specs");
    config.generation.include_devops_section = Confirm::new()
        .with_prompt("Include DevOps section?")
        .default(config.generation.include_devops_section)
        .interact()?;

    print_desc("Include accessibility requirements in generated specs");
    config.generation.include_accessibility = Confirm::new()
        .with_prompt("Include accessibility?")
        .default(config.generation.include_accessibility)
        .interact()?;

    print_desc("Include future enhancements section in generated specs");
    config.generation.include_future_enhancements = Confirm::new()
        .with_prompt("Include future enhancements?")
        .default(config.generation.include_future_enhancements)
        .interact()?;

    // ─────────────────────────────────────────────────────────────────────────
    // SECURITY SECTION
    // ─────────────────────────────────────────────────────────────────────────
    print_section("Security", "Configure security constraints for the autonomous agent");

    print_desc("Strictly enforce the security allowlist (block dangerous commands)");
    config.security.enforce_allowlist = Confirm::new()
        .with_prompt("Enforce allowlist?")
        .default(config.security.enforce_allowlist)
        .interact()?;

    print_desc("Path to security allowlist JSON file");
    config.security.allowlist_file = Input::new()
        .with_prompt("Allowlist file")
        .default(config.security.allowlist_file.clone())
        .interact_text()?;

    // ─────────────────────────────────────────────────────────────────────────
    // NOTIFICATIONS SECTION
    // ─────────────────────────────────────────────────────────────────────────
    print_section(
        "Notifications",
        "Configure webhook notifications for feature completions",
    );

    print_desc("Send webhook notifications when features are completed (e.g., to Discord)");
    config.notifications.webhook_enabled = Confirm::new()
        .with_prompt("Enable webhook notifications?")
        .default(config.notifications.webhook_enabled)
        .interact()?;

    if config.notifications.webhook_enabled {
        print_desc("Webhook URL to POST notifications to (Discord/Slack compatible)");
        let current_url = config.notifications.webhook_url.clone().unwrap_or_default();
        let url: String = Input::new()
            .with_prompt("Webhook URL")
            .default(current_url)
            .interact_text()?;
        config.notifications.webhook_url = if url.is_empty() { None } else { Some(url) };
    }

    // ─────────────────────────────────────────────────────────────────────────
    // UI SECTION
    // ─────────────────────────────────────────────────────────────────────────
    print_section("UI Settings", "Configure display and output preferences");

    print_desc("Show colored terminal output (disable for plain text logs)");
    config.ui.colored_output = Confirm::new()
        .with_prompt("Colored output?")
        .default(config.ui.colored_output)
        .interact()?;

    print_desc("Show verbose debug output");
    config.ui.verbose = Confirm::new()
        .with_prompt("Verbose mode?")
        .default(config.ui.verbose)
        .interact()?;

    print_desc("Show progress indicators during long operations");
    config.ui.show_progress = Confirm::new()
        .with_prompt("Show progress?")
        .default(config.ui.show_progress)
        .interact()?;

    print_desc("Number of lines to show in spec preview");
    config.ui.spec_preview_lines = Input::new()
        .with_prompt("Spec preview lines")
        .default(config.ui.spec_preview_lines)
        .interact()?;

    // ─────────────────────────────────────────────────────────────────────────
    // SAVE CONFIG
    // ─────────────────────────────────────────────────────────────────────────
    save_config(&config, config_path)?;

    println!(
        "\n{}",
        style("✅ Configuration saved to autocode.toml")
            .green()
            .bold()
    );

    // Print next steps
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

    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Helper Functions
// ─────────────────────────────────────────────────────────────────────────────

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

fn print_section(title: &str, description: &str) {
    println!("\n{}", style("═".repeat(55)).yellow());
    println!("{}", style(format!("  {}", title)).yellow().bold());
    println!("{}", style(format!("  {}", description)).dim());
    println!("{}", style("═".repeat(55)).yellow());
}

fn print_desc(description: &str) {
    println!("\n{}", style(format!("  ℹ {}", description)).dim());
}

/// Fetch available models from opencode
pub fn fetch_available_models() -> Result<Vec<String>> {
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
    // Find current model index or add it
    let mut options: Vec<String> = models.to_vec();
    let current_idx = options.iter().position(|m| m == current);

    // Add current if not in list
    if current_idx.is_none() && !current.is_empty() {
        options.insert(0, current.to_string());
    }

    // Add custom option
    options.push("(Enter custom model)".to_string());

    let default_idx = options.iter().position(|m| m == current).unwrap_or(0);

    let idx = FuzzySelect::new()
        .with_prompt(prompt)
        .items(&options)
        .default(default_idx)
        .max_length(15)
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

/// Select from a fixed list of options
fn select_from_list(prompt: &str, options: &[&str], current: &str) -> Result<String> {
    let default_idx = options
        .iter()
        .position(|&o| o.eq_ignore_ascii_case(current))
        .unwrap_or(0);

    let idx = Select::new()
        .with_prompt(prompt)
        .items(options)
        .default(default_idx)
        .interact()?;

    Ok(options[idx].to_string())
}

/// Parse comma-separated list into Vec<String>
fn parse_comma_list(input: &str) -> Vec<String> {
    input
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Save config to TOML file
fn save_config(config: &Config, path: &Path) -> Result<()> {
    let content = format!(
        r#"# OpenCode Autocode Configuration
# Generated by opencode-autocode --config

# ─────────────────────────────────────────────────────────────────────────────
# Models - AI models used for different tasks
# ─────────────────────────────────────────────────────────────────────────────
[models]
default = "{}"      # Spec generation
autonomous = "{}"   # Vibe loop coding
reasoning = "{}"    # Complex planning
enhancement = "{}"  # Enhancement discovery

# ─────────────────────────────────────────────────────────────────────────────
# Autonomous Loop - Control vibe loop behavior
# ─────────────────────────────────────────────────────────────────────────────
[autonomous]
max_iterations = {}           # 0 = unlimited
delay_between_sessions = {}   # Seconds between iterations
session_timeout_minutes = {}  # 0 = no timeout
auto_commit = {}              # Commit on feature completion
log_level = "{}"

# ─────────────────────────────────────────────────────────────────────────────
# Agent Behavior - Fine-tune agent approach
# ─────────────────────────────────────────────────────────────────────────────
[agent]
max_retry_attempts = {}       # Retries before research mode
max_research_attempts = {}    # Research attempts before giving up
verification_sample_size = {} # Regression check sample size
single_feature_focus = {}     # Focus on one feature at a time

# ─────────────────────────────────────────────────────────────────────────────
# Alternative Approaches - Stuck recovery configuration
# ─────────────────────────────────────────────────────────────────────────────
[alternative_approaches]
enabled = {}
num_approaches = {}
retry_threshold = {}
cache_results = {}
cache_dir = "{}"

# ─────────────────────────────────────────────────────────────────────────────
# MCP Tools - Model Context Protocol configuration
# ─────────────────────────────────────────────────────────────────────────────
[mcp]
prefer_osgrep = {}
use_sequential_thinking = {}
required_tools = [{}]

# ─────────────────────────────────────────────────────────────────────────────
# Conductor - Context-driven planning
# ─────────────────────────────────────────────────────────────────────────────
[conductor]
context_dir = "{}"
tracks_dir = "{}"
auto_setup = {}
planning_mode = "{}"
checkpoint_frequency = {}

# ─────────────────────────────────────────────────────────────────────────────
# Generation - Spec generation preferences
# ─────────────────────────────────────────────────────────────────────────────
[generation]
complexity = "{}"
include_security_section = {}
include_testing_strategy = {}
include_devops_section = {}
include_accessibility = {}
include_future_enhancements = {}

# ─────────────────────────────────────────────────────────────────────────────
# Security - Agent security constraints
# ─────────────────────────────────────────────────────────────────────────────
[security]
enforce_allowlist = {}
allowlist_file = "{}"

# ─────────────────────────────────────────────────────────────────────────────
# Paths - File and directory locations
# ─────────────────────────────────────────────────────────────────────────────
[paths]
feature_list_file = "{}"
progress_file = "{}"
log_dir = "{}"

# ─────────────────────────────────────────────────────────────────────────────
# Notifications - Webhook configuration
# ─────────────────────────────────────────────────────────────────────────────
[notifications]
webhook_enabled = {}
{}

# ─────────────────────────────────────────────────────────────────────────────
# UI - Display preferences
# ─────────────────────────────────────────────────────────────────────────────
[ui]
colored_output = {}
verbose = {}
show_progress = {}
spec_preview_lines = {}
"#,
        config.models.default,
        config.models.autonomous,
        config.models.reasoning,
        config.models.enhancement,
        config.autonomous.max_iterations,
        config.autonomous.delay_between_sessions,
        config.autonomous.session_timeout_minutes,
        config.autonomous.auto_commit,
        config.autonomous.log_level,
        config.agent.max_retry_attempts,
        config.agent.max_research_attempts,
        config.agent.verification_sample_size,
        config.agent.single_feature_focus,
        config.alternative_approaches.enabled,
        config.alternative_approaches.num_approaches,
        config.alternative_approaches.retry_threshold,
        config.alternative_approaches.cache_results,
        config.alternative_approaches.cache_dir,
        config.mcp.prefer_osgrep,
        config.mcp.use_sequential_thinking,
        config.mcp.required_tools.iter().map(|t| format!("\"{}\"", t)).collect::<Vec<_>>().join(", "),
        config.conductor.context_dir,
        config.conductor.tracks_dir,
        config.conductor.auto_setup,
        config.conductor.planning_mode,
        config.conductor.checkpoint_frequency,
        config.generation.complexity,
        config.generation.include_security_section,
        config.generation.include_testing_strategy,
        config.generation.include_devops_section,
        config.generation.include_accessibility,
        config.generation.include_future_enhancements,
        config.security.enforce_allowlist,
        config.security.allowlist_file,
        config.paths.feature_list_file,
        config.paths.progress_file,
        config.paths.log_dir,
        config.notifications.webhook_enabled,
        match &config.notifications.webhook_url {
            Some(url) => format!("webhook_url = \"{}\"", url),
            None => "# webhook_url = \"\"".to_string(),
        },
        config.ui.colored_output,
        config.ui.verbose,
        config.ui.show_progress,
        config.ui.spec_preview_lines,
    );

    fs::write(path, content).context("Failed to write autocode.toml")?;
    Ok(())
}
