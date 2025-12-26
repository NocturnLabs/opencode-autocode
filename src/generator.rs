//! AI-based spec generation using OpenCode CLI
//!
//! This module provides functionality to generate project specifications
//! from a user's idea by leveraging OpenCode's LLM capabilities.

use anyhow::{bail, Context, Result};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::time::Instant;

use crate::config::Config;

/// Embedded prompt template for spec generation (legacy single-pass)
const GENERATOR_PROMPT: &str = include_str!("../templates/generator_prompt.md");

/// Embedded prompt template for subagent-based parallel generation
const SUBAGENT_PROMPT: &str = include_str!("../templates/generator/subagent_prompt.md");

/// Embedded prompt template for spec refinement
const REFINE_PROMPT: &str = include_str!("../templates/refine_prompt.md");

/// Generate a project specification from a user's idea using OpenCode CLI.
///
/// This function shells out to `opencode run` with a carefully crafted prompt
/// that instructs the LLM to research the idea and generate a comprehensive
/// project specification in XML format.
///
/// # Arguments
/// * `idea` - The user's project idea description
/// * `model` - Optional model to use (defaults to configured or big-pickle)
/// * `on_output` - Callback for streaming output lines to the user
///
/// # Returns
/// The generated specification text (XML format)
pub fn generate_spec_from_idea<F>(
    idea: &str,
    model: Option<&str>,
    mut on_output: F,
) -> Result<String>
where
    F: FnMut(&str),
{
    // Load config
    let config = Config::load(None).unwrap_or_default();

    // Build the prompt with the user's idea (use subagents if enabled)
    let prompt = if config.generation.enable_subagents {
        build_subagent_prompt(idea)
    } else {
        build_generation_prompt(idea)
    };

    // Check if opencode is available
    let opencode_path = which_opencode(&config)?;

    let model_to_use = model.unwrap_or(&config.models.default);
    
    // Performance timing
    let start_time = Instant::now();
    if config.generation.enable_subagents {
        on_output(&format!(
            "[PERF] Starting subagent spec generation at {:?}\n",
            std::time::SystemTime::now()
        ));
        on_output(&format!(
            "🚀 Generating spec with parallel subagents (model: {})...\n",
            model_to_use
        ));
    } else {
        on_output(&format!(
            "🔍 Researching your idea with OpenCode (model: {})...\n",
            model_to_use
        ));
    }
    on_output("   (This may take a minute as the AI researches best practices)\n\n");

    // Run opencode with the prompt
    let mut child = Command::new(&opencode_path)
        .args(["run", "--model", model_to_use, &prompt])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to spawn opencode at: {}", opencode_path))?;

    // Capture output
    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let reader = BufReader::new(stdout);

    let mut full_output = String::new();
    for line in reader.lines() {
        match line {
            Ok(line) => {
                full_output.push_str(&line);
                full_output.push('\n');
                // Stream progress to user (but don't overwhelm them)
                if line.contains("Searching") || line.contains("Reading") || line.contains("Tool") {
                    on_output(&format!("   {}\n", line));
                }
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
            "OpenCode exited with error. Output:\n{}",
            full_output.chars().take(1000).collect::<String>()
        );
    }

    // Extract the XML specification from the output
    let spec = extract_spec_from_output(&full_output)?;

    // Log performance timing
    let elapsed = start_time.elapsed();
    on_output(&format!(
        "[PERF] Spec generation completed in {:.2}s\n",
        elapsed.as_secs_f64()
    ));

    Ok(spec)
}

/// Refine an existing specification based on user feedback using OpenCode CLI.
///
/// # Arguments
/// * `current_spec` - The current XML specification
/// * `refinement` - User's refinement instructions
/// * `model` - Optional model to use
/// * `on_output` - Callback for streaming output lines to the user
///
/// # Returns
/// The refined specification text (XML format)
pub fn refine_spec_from_idea<F>(
    current_spec: &str,
    refinement: &str,
    model: Option<&str>,
    mut on_output: F,
) -> Result<String>
where
    F: FnMut(&str),
{
    // Load config
    let config = Config::load(None).unwrap_or_default();

    // Build the refinement prompt
    let prompt = build_refine_prompt(current_spec, refinement);

    // Check if opencode is available
    let opencode_path = which_opencode(&config)?;

    let model_to_use = model.unwrap_or(&config.models.default);
    on_output(&format!(
        "🔧 Refining specification with OpenCode (model: {})...\n",
        model_to_use
    ));
    on_output("   (Applying your refinement instructions)\n\n");

    // Run opencode with the prompt
    let mut child = Command::new(&opencode_path)
        .args(["run", "--model", model_to_use, &prompt])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to spawn opencode at: {}", opencode_path))?;

    // Capture output
    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let reader = BufReader::new(stdout);

    let mut full_output = String::new();
    for line in reader.lines() {
        match line {
            Ok(line) => {
                full_output.push_str(&line);
                full_output.push('\n');
                // Stream progress to user
                if line.contains("Searching") || line.contains("Reading") || line.contains("Tool") {
                    on_output(&format!("   {}\n", line));
                }
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
            "OpenCode exited with error. Output:\n{}",
            full_output.chars().take(1000).collect::<String>()
        );
    }

    // Extract the XML specification from the output
    let spec = extract_spec_from_output(&full_output)?;

    Ok(spec)
}

/// Build the generation prompt by inserting the user's idea into the template.
fn build_generation_prompt(idea: &str) -> String {
    GENERATOR_PROMPT.replace("{{IDEA}}", idea)
}

/// Build the subagent-based generation prompt by inserting the user's idea.
fn build_subagent_prompt(idea: &str) -> String {
    SUBAGENT_PROMPT.replace("{{IDEA}}", idea)
}

/// Build the refinement prompt by inserting the current spec and refinement instructions.
fn build_refine_prompt(current_spec: &str, refinement: &str) -> String {
    REFINE_PROMPT
        .replace("{{EXISTING_SPEC}}", current_spec)
        .replace("{{REFINEMENT}}", refinement)
}

/// Find the opencode executable using paths from config.
fn which_opencode(config: &Config) -> Result<String> {
    // Try paths from config first
    for candidate in &config.paths.opencode_paths {
        if Command::new("which")
            .arg(candidate)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            return Ok(candidate.to_string());
        }

        // Also try direct execution test
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
         Searched paths: {:?}\n\
         Install it from: https://opencode.ai",
        config.paths.opencode_paths
    )
}

/// Extract the project specification XML from OpenCode's output.
///
/// The output may contain various messages, tool call results, etc.
/// We need to find the XML specification block.
fn extract_spec_from_output(output: &str) -> Result<String> {
    // Look for the XML specification block
    // It should start with <project_specification> and end with </project_specification>

    if let Some(start) = output.find("<project_specification>") {
        if let Some(end) = output.find("</project_specification>") {
            let spec = &output[start..end + "</project_specification>".len()];
            return Ok(spec.to_string());
        }
    }

    // Try to find it in markdown code blocks
    if let Some(start) = output.find("```xml") {
        if let Some(end) = output[start..].find("```\n") {
            let block = &output[start + 6..start + end];
            if block.contains("<project_specification>") {
                return Ok(block.trim().to_string());
            }
        }
    }

    // If we can't find XML, maybe the output IS the spec (just wrapped differently)
    if output.contains("<project_name>") && output.contains("<overview>") {
        // Try to reconstruct from fragments
        bail!(
            "Could not extract complete specification. \
             The AI response may be malformed. Please try again."
        );
    }

    bail!(
        "No project specification found in OpenCode output. \
         The AI may have encountered an error or produced unexpected output.\n\n\
         Partial output:\n{}",
        output.chars().take(500).collect::<String>()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_generation_prompt() {
        let idea = "A todo app with tags";
        let prompt = build_generation_prompt(idea);

        assert!(prompt.contains("A todo app with tags"));
        assert!(prompt.contains("<project_specification>"));
        assert!(!prompt.contains("{{IDEA}}"));
    }

    #[test]
    fn test_extract_spec_from_output() {
        let output = r#"
Some preamble text...

<project_specification>
<project_name>Test Project</project_name>
<overview>A test</overview>
</project_specification>

Some trailing text...
"#;

        let spec = extract_spec_from_output(output).unwrap();
        assert!(spec.starts_with("<project_specification>"));
        assert!(spec.ends_with("</project_specification>"));
        assert!(spec.contains("Test Project"));
    }

    #[test]
    fn test_extract_spec_no_match() {
        let output = "This is just random text without any spec";
        let result = extract_spec_from_output(output);
        assert!(result.is_err());
    }
}
