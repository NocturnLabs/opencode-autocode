//! Model selection utilities for config TUI

use anyhow::{Context, Result};
use dialoguer::{FuzzySelect, Input, Select};
use std::process::Command;

/// Fetch available models from opencode CLI
pub fn fetch_available_models() -> Result<Vec<String>> {
    let output = Command::new("opencode")
        .arg("models")
        .output()
        .context("Failed to run 'opencode models'. Is OpenCode installed?")?;

    if !output.status.success() {
        // Fallback to common models if command fails
        return Ok(vec![
            "opencode/glm-4.7-free".to_string(),
            "opencode/minimax-m2.1-free".to_string(),
            "opencode/big-pickle".to_string(),
            "opencode/grok-code".to_string(),
        ]);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let models: Vec<String> = stdout
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if models.is_empty() {
        Ok(vec![
            "anthropic/claude-sonnet-4-20250514".to_string(),
            "google/gemini-2.5-pro".to_string(),
        ])
    } else {
        Ok(models)
    }
}

/// Prompt user to select a model from available options or enter custom
pub fn prompt_model_selection(prompt: &str, models: &[String], current: &str) -> Result<String> {
    let mut options: Vec<String> = models.to_vec();

    // Ensure current model is in the list and at the top
    if !current.is_empty() {
        // Remove if already exists to avoid duplicates
        options.retain(|m| m != current);
        // Insert at the top
        options.insert(0, current.to_string());
    }

    let mut display_options: Vec<&str> = options.iter().map(|s| s.as_str()).collect();
    display_options.push("(enter custom)");

    // Always start at top of list (which is the current model)
    let selection = FuzzySelect::new()
        .with_prompt(prompt)
        .items(&display_options)
        .default(0)
        .max_length(5)
        .interact()?;

    if display_options[selection] == "(enter custom)" {
        let custom: String = Input::new()
            .with_prompt("Custom model ID")
            .default(current.to_string())
            .interact_text()?;
        Ok(custom)
    } else {
        Ok(options[selection].to_string())
    }
}

/// Prompt user to select from a fixed list of options
pub fn prompt_list_selection(prompt: &str, options: &[&str], current: &str) -> Result<String> {
    let default_idx = options
        .iter()
        .position(|&s| s.eq_ignore_ascii_case(current))
        .unwrap_or(0);

    let selection = Select::new()
        .with_prompt(prompt)
        .items(options)
        .default(default_idx)
        .interact()?;

    Ok(options[selection].to_string())
}

/// Parse a comma-separated string into a Vec<String>
pub fn parse_comma_list(input: &str) -> Vec<String> {
    input
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}
