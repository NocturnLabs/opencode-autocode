//! Model selection utilities for config TUI

use anyhow::{Context, Result};
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
