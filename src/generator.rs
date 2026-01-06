//! AI-based spec generation using OpenCode CLI
//!
//! We handle the translation of a user's raw idea into a structured project specification.
//! Our role is to orchestrate the OpenCode LLM, manage the refinement loop, and ensure
//! the final output is valid XML that the scaffolder can consume.

use anyhow::{bail, Context, Result};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::time::Instant;

use crate::config::Config;

/// Embedded prompt template for spec generation (legacy single-pass)
use crate::validation;

/// Embedded prompt template for spec generation (legacy single-pass)
const GENERATOR_PROMPT: &str = include_str!("../templates/generator_prompt.md");

/// Embedded prompt template for subagent-based parallel generation
const SUBAGENT_PROMPT: &str = include_str!("../templates/generator/subagent_prompt.md");

/// Embedded prompt template for spec refinement
const REFINE_PROMPT: &str = include_str!("../templates/refine_prompt.md");

/// Embedded prompt template for fixing malformed XML
const FIX_MALFORMED_PROMPT: &str = include_str!("../templates/generator/fix_malformed_xml.md");

/// Generate a project specification from a user's idea using OpenCode CLI.
///
/// We shell out to `opencode run` with a carefully crafted prompt that instructs
/// the LLM to research the idea. We then monitor the output, capturing the
/// generated XML and validating it against our schema.
///
/// # Arguments
/// * `idea` - The user's project idea description
/// * `testing_preference` - Optional testing framework preference
/// * `model` - Optional model to use (defaults to configured or big-pickle)
/// * `use_subagents` - Whether to use parallel subagent generation
/// * `on_output` - Callback for streaming output lines to the user
///
/// # Returns
/// The generated specification text (XML format)
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
        build_subagent_prompt(idea, testing_preference, config)
    } else {
        build_generation_prompt(idea, testing_preference, config)
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
        // Why? The default model (often a cheaper reasoning model) might be stuck in a specific
        // failure mode. The fixer model (e.g., grok-code) is chosen specifically for its serialization
        // reliability, which is critical when repairing malformed XML.
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
            // We might try again?
            if attempt == max_retries {
                bail!(
                    "OpenCode exited with error. Output:\n{}",
                    full_output.chars().take(1000).collect::<String>()
                );
            }
            continue;
        }

        // We'll try to extract the XML block and then run it through our strict validator.
        // If validation fails, we don't just give up—we feed the specific validation error
        // back into the prompt so the next attempt can self-correct.
        match extract_spec_from_output(&full_output) {
            Ok(spec) => {
                // Validate XML structure
                match validation::validate_spec(&spec) {
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
                            prompt = build_fix_prompt(idea, &last_error, Some(&full_output));
                        }
                    }
                    Err(e) => {
                        // Validator crashed?
                        last_error = format!("Validator error: {}", e);
                        prompt = build_fix_prompt(idea, &last_error, Some(&full_output));
                    }
                }
            }
            Err(e) => {
                // Could not extract XML block
                last_error = format!("Could not extract XML: {}", e);
                prompt = build_fix_prompt(
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
    config: &Config,
    mut on_output: F,
) -> Result<String>
where
    F: FnMut(&str),
{
    // Build the refinement prompt
    let prompt = build_refine_prompt(current_spec, refinement);

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
    let spec = extract_spec_from_output(&full_output)?;

    Ok(spec)
}

/// Build the generation prompt by inserting the user's idea and configuration constraints.
fn build_generation_prompt(
    idea: &str,
    testing_preference: Option<&str>,
    config: &Config,
) -> String {
    let pref_text = match testing_preference {
        Some(pref) if !pref.trim().is_empty() => format!(
            "\n## User Preferences\n\nTesting & QA Framework Preference: {}\n",
            pref
        ),
        _ => String::new(),
    };

    let guidance = if config.generation.complexity == "minimal" {
        "The target is a minimal, lightweight implementation. Focus only on the absolute core."
    } else {
        "The target is a comprehensive, production-ready specification with deep detail."
    };

    GENERATOR_PROMPT
        .replace("{{IDEA}}", idea)
        .replace("{{TESTING_PREFERENCE}}", &pref_text)
        .replace("{{COMPLEXITY_GUIDANCE}}", guidance)
}

/// Build the subagent-based generation prompt by inserting the user's idea and constraints.
fn build_subagent_prompt(idea: &str, testing_preference: Option<&str>, _config: &Config) -> String {
    let pref_text = match testing_preference {
        Some(pref) if !pref.trim().is_empty() => format!(
            "\n**User Preference:** QA/Testing framework should be: {}\n",
            pref
        ),
        _ => String::new(),
    };

    SUBAGENT_PROMPT
        .replace("{{IDEA}}", idea)
        .replace("{{TESTING_PREFERENCE}}", &pref_text)
        .replace("{{BLUEPRINT}}", "[The blueprint you generated above]")
}

/// Build the refinement prompt by inserting the current spec and refinement instructions.
fn build_refine_prompt(current_spec: &str, refinement: &str) -> String {
    REFINE_PROMPT
        .replace("{{EXISTING_SPEC}}", current_spec)
        .replace("{{REFINEMENT}}", refinement)
}

/// Build the fix prompt by inserting the original idea and error message.
fn build_fix_prompt(idea: &str, errors: &str, partial_output: Option<&str>) -> String {
    let fix_prompt = FIX_MALFORMED_PROMPT
        .replace("{{IDEA}}", idea)
        .replace("{{ERRORS}}", errors);

    let partial_text = match partial_output {
        Some(output) if !output.trim().is_empty() => {
            let truncated = if output.len() > 10000 {
                format!("... (truncated) ...\n{}", &output[output.len() - 10000..])
            } else {
                output.to_string()
            };
            format!(
                "\n## Partial Output (for context)\n\n```\n{}\n```\n",
                truncated
            )
        }
        _ => String::new(),
    };

    fix_prompt.replace("{{PARTIAL_OUTPUT}}", &partial_text)
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
        let config = Config::default();
        let prompt = build_generation_prompt(idea, None, &config);

        assert!(prompt.contains("A todo app with tags"));
        assert!(prompt.contains("<project_specification>"));
        assert!(!prompt.contains("{{IDEA}}"));
        // Count placeholders should no longer be present since we removed them
    }

    #[test]
    fn test_build_generation_prompt_contains_constraints() {
        let idea = "A complex ERP";
        let config = Config::default();
        let prompt = build_generation_prompt(idea, None, &config);

        // With counts removed, we now test for the qualitative complexity guidance
        assert!(prompt.contains("production-ready specification with deep detail"));
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
