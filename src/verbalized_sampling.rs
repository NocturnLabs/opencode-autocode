#![allow(dead_code, clippy::too_many_arguments)]
//! Verbalized Sampling implementation for diverse approach generation
//!
//! This module implements the Verbalized Sampling technique to overcome LLM mode collapse
//! and unlock creative implementation approaches. It generates diverse approaches using
//! a primary model, then uses a context-aware selector model to choose the best fit.

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};

use crate::config::Config;

/// Embedded prompt template for verbalized sampling
const VS_PROMPT: &str = include_str!("../templates/verbalized_sampling_prompt.md");

/// Embedded prompt template for approach selection
const SELECTOR_PROMPT: &str = include_str!("../templates/approach_selector_prompt.md");

/// Default selector model (DeepSeek v3.2 on io-intelligence)
const DEFAULT_SELECTOR_MODEL: &str = "io-intelligence/deepseek-ai/DeepSeek-V3.2";

/// A single implementation approach with probability and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApproachResponse {
    pub text: String,
    pub probability: f32,
    pub reasoning: String,
    #[serde(default)]
    pub key_techniques: Vec<String>,
    #[serde(default)]
    pub trade_offs: String,
}

/// Result of verbalized sampling - contains all generated approaches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerbalizationResult {
    pub responses: Vec<ApproachResponse>,
    #[serde(skip)]
    pub feature_hash: String,
}

/// Result of approach selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApproachSelection {
    pub selected_index: usize,
    pub approach: String,
    #[serde(default)]
    pub probability_override_reason: String,
    pub justification: String,
    #[serde(default)]
    pub alignment_scores: AlignmentScores,
    #[serde(default)]
    pub adaptations: Vec<String>,
    #[serde(default)]
    pub implementation_notes: String,
}

/// Alignment scores from the selector
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AlignmentScores {
    #[serde(default)]
    pub project_alignment: u8,
    #[serde(default)]
    pub codebase_consistency: u8,
    #[serde(default)]
    pub technical_fit: u8,
    #[serde(default)]
    pub feature_requirements: u8,
    #[serde(default)]
    pub maintainability: u8,
}

/// Configuration for Verbalized Sampling
#[derive(Debug, Clone)]
pub struct VSConfig {
    /// Model to use for generating approaches
    pub generator_model: Option<String>,
    /// Model to use for selecting approaches (defaults to DeepSeek v3.2)
    pub selector_model: String,
    /// Directory to cache VS results
    pub cache_dir: String,
    /// Whether caching is enabled
    pub cache_enabled: bool,
}

impl Default for VSConfig {
    fn default() -> Self {
        Self {
            generator_model: None,
            selector_model: DEFAULT_SELECTOR_MODEL.to_string(),
            cache_dir: ".vs-cache".to_string(),
            cache_enabled: true,
        }
    }
}

/// Generate a hash for the feature to use as cache key
fn feature_hash(feature_description: &str) -> String {
    let mut hasher = DefaultHasher::new();
    feature_description.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// Check if cached approaches exist for a feature
pub fn get_cached_approaches(
    feature_description: &str,
    cache_dir: &str,
) -> Option<VerbalizationResult> {
    let hash = feature_hash(feature_description);
    let cache_path = Path::new(cache_dir).join(format!("{}.json", hash));

    if cache_path.exists() {
        if let Ok(content) = fs::read_to_string(&cache_path) {
            if let Ok(mut result) = serde_json::from_str::<VerbalizationResult>(&content) {
                result.feature_hash = hash;
                return Some(result);
            }
        }
    }
    None
}

/// Save approaches to cache
pub fn cache_approaches(
    feature_description: &str,
    result: &VerbalizationResult,
    cache_dir: &str,
) -> Result<()> {
    let hash = feature_hash(feature_description);
    let cache_dir_path = Path::new(cache_dir);

    // Create cache directory if it doesn't exist
    fs::create_dir_all(cache_dir_path)?;

    let cache_path = cache_dir_path.join(format!("{}.json", hash));
    let json = serde_json::to_string_pretty(result)?;
    fs::write(cache_path, json)?;

    Ok(())
}

/// Generate diverse implementation approaches using Verbalized Sampling
///
/// # Arguments
/// * `feature` - Description of the feature to implement
/// * `tech_stack` - Technology stack being used
/// * `project_context` - Context about the project
/// * `codebase_patterns` - Existing patterns in the codebase
/// * `vs_config` - Configuration for VS
/// * `on_output` - Callback for streaming output
///
/// # Returns
/// A VerbalizationResult containing 10 diverse approaches
pub fn generate_approaches<F>(
    feature: &str,
    tech_stack: &str,
    project_context: &str,
    codebase_patterns: &str,
    vs_config: &VSConfig,
    mut on_output: F,
) -> Result<VerbalizationResult>
where
    F: FnMut(&str),
{
    // Check cache first if enabled
    if vs_config.cache_enabled {
        if let Some(cached) = get_cached_approaches(feature, &vs_config.cache_dir) {
            on_output("📦 Using cached VS approaches\n");
            return Ok(cached);
        }
    }

    // Load config for opencode path
    let config = Config::load(None).unwrap_or_default();

    // Build the VS prompt
    let prompt = VS_PROMPT
        .replace("{{FEATURE_DESCRIPTION}}", feature)
        .replace("{{TECH_STACK}}", tech_stack)
        .replace("{{PROJECT_CONTEXT}}", project_context)
        .replace("{{CODEBASE_PATTERNS}}", codebase_patterns);

    // Find opencode
    let opencode_path = which_opencode(&config)?;

    let model = vs_config
        .generator_model
        .as_deref()
        .unwrap_or(&config.models.default);

    on_output(&format!(
        "🎲 Generating diverse approaches with Verbalized Sampling (model: {})...\n",
        model
    ));

    // Run opencode with the prompt
    let mut child = Command::new(&opencode_path)
        .args(["run", "--model", model, &prompt])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to spawn opencode at: {}", opencode_path))?;

    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let reader = BufReader::new(stdout);

    let mut full_output = String::new();
    for line in reader.lines() {
        match line {
            Ok(line) => {
                full_output.push_str(&line);
                full_output.push('\n');
            }
            Err(e) => {
                on_output(&format!("   Warning: Error reading output: {}\n", e));
            }
        }
    }

    let status = child
        .wait()
        .context("Failed to wait for opencode process")?;

    if !status.success() {
        bail!(
            "OpenCode exited with error during VS. Output:\n{}",
            full_output.chars().take(1000).collect::<String>()
        );
    }

    // Parse the JSON response
    let result = extract_verbalization_result(&full_output)?;

    // Cache the result
    if vs_config.cache_enabled {
        if let Err(e) = cache_approaches(feature, &result, &vs_config.cache_dir) {
            on_output(&format!("   Warning: Failed to cache VS results: {}\n", e));
        }
    }

    on_output(&format!(
        "✅ Generated {} approaches (cache key: {})\n",
        result.responses.len(),
        feature_hash(feature)
    ));

    Ok(result)
}

/// Select the best approach based on project context
///
/// # Arguments
/// * `approaches` - The generated approaches to select from
/// * `project_overview` - Overview of the project
/// * `tech_stack` - Technology stack
/// * `codebase_patterns` - Existing patterns
/// * `feature_description` - The feature being implemented
/// * `feature_priority` - Priority level (high/medium/low)
/// * `risk_level` - Risk tolerance (conservative/moderate/experimental)
/// * `vs_config` - Configuration for VS
/// * `on_output` - Callback for streaming output
///
/// # Returns
/// The selected approach with justification
pub fn select_approach<F>(
    approaches: &VerbalizationResult,
    project_overview: &str,
    tech_stack: &str,
    codebase_patterns: &str,
    feature_description: &str,
    feature_priority: &str,
    risk_level: &str,
    vs_config: &VSConfig,
    mut on_output: F,
) -> Result<ApproachSelection>
where
    F: FnMut(&str),
{
    let config = Config::load(None).unwrap_or_default();

    // Serialize approaches to JSON for the prompt
    let approaches_json = serde_json::to_string_pretty(&approaches.responses)?;

    // Build the selector prompt
    let prompt = SELECTOR_PROMPT
        .replace("{{PROJECT_OVERVIEW}}", project_overview)
        .replace("{{TECH_STACK}}", tech_stack)
        .replace("{{CODEBASE_PATTERNS}}", codebase_patterns)
        .replace("{{FEATURE_DESCRIPTION}}", feature_description)
        .replace("{{FEATURE_PRIORITY}}", feature_priority)
        .replace("{{RISK_LEVEL}}", risk_level)
        .replace("{{APPROACHES_JSON}}", &approaches_json);

    let opencode_path = which_opencode(&config)?;

    on_output(&format!(
        "🎯 Selecting best approach with context-aware model ({})...\n",
        vs_config.selector_model
    ));

    // Run opencode with selector model
    let mut child = Command::new(&opencode_path)
        .args(["run", "--model", &vs_config.selector_model, &prompt])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to spawn opencode at: {}", opencode_path))?;

    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let reader = BufReader::new(stdout);

    let mut full_output = String::new();
    for line in reader.lines() {
        match line {
            Ok(line) => {
                full_output.push_str(&line);
                full_output.push('\n');
            }
            Err(e) => {
                on_output(&format!("   Warning: Error reading output: {}\n", e));
            }
        }
    }

    let status = child
        .wait()
        .context("Failed to wait for opencode process")?;

    if !status.success() {
        bail!(
            "OpenCode exited with error during approach selection. Output:\n{}",
            full_output.chars().take(1000).collect::<String>()
        );
    }

    // Parse the selection result
    let selection = extract_selection_result(&full_output)?;

    on_output(&format!(
        "✅ Selected approach #{} (probability: {:.2})\n",
        selection.selected_index + 1,
        approaches
            .responses
            .get(selection.selected_index)
            .map(|a| a.probability)
            .unwrap_or(0.0)
    ));

    Ok(selection)
}

/// Calculate a hybrid score for an approach combining context fit and diversity
pub fn hybrid_score(
    approach: &ApproachResponse,
    context_fit: f32,
    historical_success: Option<f32>,
) -> f32 {
    let diversity_bonus = 1.0 - approach.probability;
    let history_weight = historical_success.unwrap_or(0.5);

    0.5 * context_fit + 0.3 * diversity_bonus + 0.2 * history_weight
}

/// Extract VerbalizationResult from OpenCode output
fn extract_verbalization_result(output: &str) -> Result<VerbalizationResult> {
    // Try to find JSON in the output
    if let Some(start) = output.find("{") {
        if let Some(end) = output.rfind("}") {
            let json_str = &output[start..=end];
            if let Ok(result) = serde_json::from_str::<VerbalizationResult>(json_str) {
                return Ok(result);
            }
        }
    }

    // Try to find JSON in code blocks
    if let Some(start) = output.find("```json") {
        if let Some(end) = output[start..].find("```\n") {
            let json_str = &output[start + 7..start + end];
            if let Ok(result) = serde_json::from_str::<VerbalizationResult>(json_str.trim()) {
                return Ok(result);
            }
        }
    }

    bail!(
        "Could not parse VS response as JSON. \
         The model may have produced malformed output.\n\n\
         Partial output:\n{}",
        output.chars().take(500).collect::<String>()
    )
}

/// Extract ApproachSelection from OpenCode output
fn extract_selection_result(output: &str) -> Result<ApproachSelection> {
    // Try to find JSON in the output
    if let Some(start) = output.find("{") {
        if let Some(end) = output.rfind("}") {
            let json_str = &output[start..=end];
            if let Ok(result) = serde_json::from_str::<ApproachSelection>(json_str) {
                return Ok(result);
            }
        }
    }

    // Try to find JSON in code blocks
    if let Some(start) = output.find("```json") {
        if let Some(end) = output[start..].find("```\n") {
            let json_str = &output[start + 7..start + end];
            if let Ok(result) = serde_json::from_str::<ApproachSelection>(json_str.trim()) {
                return Ok(result);
            }
        }
    }

    bail!(
        "Could not parse selection response as JSON. \
         The model may have produced malformed output.\n\n\
         Partial output:\n{}",
        output.chars().take(500).collect::<String>()
    )
}

/// Find the opencode executable
fn which_opencode(config: &Config) -> Result<String> {
    for candidate in &config.paths.opencode_paths {
        if Command::new("which")
            .arg(candidate)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            return Ok(candidate.to_string());
        }

        if Command::new(candidate)
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            return Ok(candidate.to_string());
        }
    }

    bail!(
        "OpenCode not found. Please ensure 'opencode' is installed and in your PATH.\n\
         Searched paths: {:?}",
        config.paths.opencode_paths
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_hash() {
        let hash1 = feature_hash("Add user authentication");
        let hash2 = feature_hash("Add user authentication");
        let hash3 = feature_hash("Add password reset");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_parse_verbalization_result() {
        let json = r#"
        {
            "responses": [
                {
                    "text": "Standard approach",
                    "probability": 0.85,
                    "reasoning": "Most common way",
                    "key_techniques": ["REST", "ORM"],
                    "trade_offs": "Simple but inflexible"
                },
                {
                    "text": "Creative approach",
                    "probability": 0.05,
                    "reasoning": "Rarely used",
                    "key_techniques": ["Event Sourcing"],
                    "trade_offs": "Complex but powerful"
                }
            ]
        }
        "#;

        let result: VerbalizationResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.responses.len(), 2);
        assert_eq!(result.responses[0].probability, 0.85);
        assert_eq!(result.responses[1].probability, 0.05);
    }

    #[test]
    fn test_hybrid_score() {
        let approach = ApproachResponse {
            text: "Test".to_string(),
            probability: 0.2, // Low probability = high diversity bonus
            reasoning: "Test".to_string(),
            key_techniques: vec![],
            trade_offs: String::new(),
        };

        // High context fit + low probability should score well
        let score = hybrid_score(&approach, 0.8, Some(0.6));
        assert!(score > 0.5);
    }

    #[test]
    fn test_parse_selection_result() {
        let json = r#"
        {
            "selected_index": 3,
            "approach": "Event-driven architecture",
            "justification": "Fits project patterns",
            "adaptations": ["Use existing message bus"]
        }
        "#;

        let result: ApproachSelection = serde_json::from_str(json).unwrap();
        assert_eq!(result.selected_index, 3);
        assert_eq!(result.approach, "Event-driven architecture");
    }
}
