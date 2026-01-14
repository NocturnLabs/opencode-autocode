use anyhow::{bail, Context, Result};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::time::Instant;

use crate::config::Config;
use crate::validation;

use super::parser;
use super::prompts;

/// Generate a project specification from a user's idea using OpenCode CLI.
///
/// We shell out to `opencode run` with a carefully crafted prompt that instructs
/// the LLM to research the idea. We then monitor the output, capturing the
/// generated XML and validating it against our schema.
pub fn generate_spec_from_idea<F>(
    idea: &str,
    testing_preference: Option<&str>,
    model: Option<&str>,
    use_subagents: bool,
    config: &Config,
    mut on_output: F,
) -> Result<String>
where
    F: FnMut(&str),
{
    let mut prompt = if use_subagents {
        prompts::build_subagent_prompt(idea, testing_preference, config)
    } else {
        prompts::build_generation_prompt(idea, testing_preference, config)
    };

    // Check if opencode is available
    let opencode_path = which_opencode(config)?;
    let model_to_use = model.unwrap_or(&config.models.default);

    // Initial message
    if use_subagents {
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

    let max_retries = 5;
    let mut last_error = String::new();

    for attempt in 0..=max_retries {
        let is_retry = attempt > 0;

        // We switch to the dedicated 'fixer' model for retries.
        let current_model = if is_retry {
            &config.models.fixer
        } else {
            model_to_use
        };

        if is_retry {
            on_output(&format!(
                "\n⚠️  Spec validation failed. Retrying with {} (attempt {}/{})...\n",
                current_model, attempt, max_retries
            ));
        }

        let start_time = Instant::now();

        // Run opencode with the prompt
        let mut child = Command::new(&opencode_path)
            .args(["run", "--model", current_model, &prompt])
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
                    // Stream progress
                    if line.contains("Searching")
                        || line.contains("Reading")
                        || line.contains("Tool")
                    {
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
            // Processing failure in opencode CLI itself
            on_output("   Warning: OpenCode CLI exited with error code.\n");
            last_error = "OpenCode CLI process failed".to_string();
            // We might try again if attempt < max_retries
            if attempt == max_retries {
                bail!(
                    "OpenCode exited with error. Output:\n{}",
                    full_output.chars().take(1000).collect::<String>()
                );
            }
            continue;
        }

        // Extract and validate XML
        match parser::extract_spec_from_output(&full_output) {
            Ok(spec) => {
                // Validate XML structure
                match validation::validate_spec_with_config(&spec, config) {
                    Ok(result) => {
                        if result.is_valid {
                            // Success!
                            let elapsed = start_time.elapsed();
                            on_output(&format!(
                                "[PERF] Spec generation completed in {:.2}s\n",
                                elapsed.as_secs_f64()
                            ));
                            return Ok(spec);
                        } else {
                            // Validation logic errors (malformed XML)
                            last_error = result.errors.join("\n");
                            on_output(&format!("   ⚠️  Validation error: {}\n", last_error));
                            // Update prompt for next attempt
                            prompt =
                                prompts::build_fix_prompt(idea, &last_error, Some(&full_output));
                        }
                    }
                    Err(e) => {
                        // Validator crashed?
                        last_error = format!("Validator error: {}", e);
                        prompt = prompts::build_fix_prompt(idea, &last_error, Some(&full_output));
                    }
                }
            }
            Err(e) => {
                // Could not extract XML block
                last_error = format!("Could not extract XML: {}", e);
                prompt = prompts::build_fix_prompt(
                    idea,
                    "Could not locate <project_specification> block in output. The response may have been truncated or lacked structured XML.",
                    Some(&full_output),
                );
            }
        }
    }

    // If we get here, retries exhausted
    bail!(
        "Failed to generate valid specification after {} retries.\nLast error: {}",
        max_retries,
        last_error
    );
}

/// Refine an existing specification based on user feedback using OpenCode CLI.
pub fn refine_spec_from_idea<F>(
    current_spec: &str,
    refinement: &str,
    model: Option<&str>,
    config: &Config,
    mut on_output: F,
) -> Result<String>
where
    F: FnMut(&str),
{
    // Build the refinement prompt
    let prompt = prompts::build_refine_prompt(current_spec, refinement);

    // Check if opencode is available
    let opencode_path = which_opencode(config)?;

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
    let spec = parser::extract_spec_from_output(&full_output)?;

    Ok(spec)
}

/// Find the opencode executable using paths from config.
pub fn which_opencode(config: &Config) -> Result<String> {
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
