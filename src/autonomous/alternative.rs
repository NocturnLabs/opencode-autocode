use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config::Config;
use crate::db::features::Feature;
use crate::services::generator::executor::which_opencode;
use crate::template_xml;

/// Embedded prompt template for alternative approach generation.
const VERBALIZED_SAMPLING_PROMPT: &str =
    include_str!("../../templates/verbalized_sampling_prompt.xml");

/// @param config Loaded configuration.
/// @param feature Active feature to recover.
/// @param error_context Optional error context for the stuck session.
/// @returns The cached result path if generation succeeded.
pub fn generate_alternative_approaches(
    config: &Config,
    feature: &Feature,
    error_context: Option<&str>,
) -> Result<PathBuf> {
    let cache_dir = resolve_cache_dir(config);
    fs::create_dir_all(&cache_dir).with_context(|| {
        format!(
            "Failed to create alternative approach cache dir: {}",
            cache_dir.display()
        )
    })?;

    let cache_file = cache_dir.join(format!("{}-approaches.json", slugify(&feature.description)));
    if config.alternative_approaches.cache_results && cache_file.exists() {
        return Ok(cache_file);
    }

    let prompt = build_prompt(config, feature, error_context)?;
    let prompt_cache_dir = resolve_vs_cache_dir(config);
    fs::create_dir_all(&prompt_cache_dir).with_context(|| {
        format!(
            "Failed to create verbalized sampling cache dir: {}",
            prompt_cache_dir.display()
        )
    })?;
    let prompt_cache_path =
        prompt_cache_dir.join(format!("{}-prompt.txt", slugify(&feature.description)));
    fs::write(&prompt_cache_path, &prompt).with_context(|| {
        format!(
            "Failed to write alternative approach prompt: {}",
            prompt_cache_path.display()
        )
    })?;

    let cache_file = cache_dir.join(format!("{}-approaches.json", slugify(&feature.description)));
    if config.alternative_approaches.cache_results && cache_file.exists() {
        return Ok(cache_file);
    }

    let prompt = build_prompt(config, feature, error_context)?;
    let prompt_cache_path = cache_dir.join(format!("{}-prompt.txt", slugify(&feature.description)));
    fs::write(&prompt_cache_path, &prompt).with_context(|| {
        format!(
            "Failed to write alternative approach prompt: {}",
            prompt_cache_path.display()
        )
    })?;

    let opencode_path = which_opencode(config)?;
    let output = Command::new(opencode_path)
        .args(["run", "--model", &config.models.reasoning, &prompt])
        .output()
        .context("Failed to run opencode for alternative approaches")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Alternative approach generation failed: {}", stderr.trim());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    fs::write(&cache_file, stdout.as_ref()).with_context(|| {
        format!(
            "Failed to write alternative approach cache: {}",
            cache_file.display()
        )
    })?;

    Ok(cache_file)
}

/// @param config Loaded configuration.
/// @returns The directory path used for caching alternative approach output.
fn resolve_cache_dir(config: &Config) -> PathBuf {
    let primary = config.alternative_approaches.cache_dir.trim();
    let fallback = config.paths.vs_cache_dir.trim();
    let selected = if primary.is_empty() {
        fallback
    } else {
        primary
    };

    PathBuf::from(selected)
}

/// @param config Loaded configuration.
/// @returns The directory path used for caching prompt inputs.
fn resolve_vs_cache_dir(config: &Config) -> PathBuf {
    let cache_dir = config.paths.vs_cache_dir.trim();
    if cache_dir.is_empty() {
        PathBuf::from(".vs-cache")
    } else {
        PathBuf::from(cache_dir)
    }
}

/// @param config Loaded configuration.
/// @param feature Active feature being recovered.
/// @param error_context Optional error context.
/// @returns The generated prompt text.
fn build_prompt(config: &Config, feature: &Feature, error_context: Option<&str>) -> Result<String> {
    let requirements = config.alternative_approaches.num_approaches.max(1);
    let app_spec_excerpt = load_app_spec_excerpt(&config.paths.app_spec_file)?;

    let prompt_template = template_xml::render_template(VERBALIZED_SAMPLING_PROMPT)
        .unwrap_or_else(|_| VERBALIZED_SAMPLING_PROMPT.to_string());

    Ok(prompt_template
        .replace("{{FEATURE_DESCRIPTION}}", &feature.description)
        .replace("{{TECH_STACK}}", "See project specification")
        .replace("{{PROJECT_CONTEXT}}", &app_spec_excerpt)
        .replace("{{FAILED_APPROACHES}}", "No recorded attempts")
        .replace(
            "{{ERROR_CONTEXT}}",
            error_context.unwrap_or("No error context available"),
        )
        .replace("{{NUM_APPROACHES}}", &requirements.to_string()))
}

/// @param app_spec_path Path to the project specification.
/// @returns A short excerpt of the project specification, if available.
fn load_app_spec_excerpt(app_spec_path: &str) -> Result<String> {
    if app_spec_path.trim().is_empty() {
        return Ok("No project specification path provided".to_string());
    }

    let path = Path::new(app_spec_path);
    if !path.exists() {
        return Ok(format!("Spec file not found at {}", app_spec_path));
    }

    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read spec file: {}", app_spec_path))?;

    Ok(content.chars().take(1200).collect())
}

/// @param input Raw feature description.
/// @returns A filesystem-safe slug.
fn slugify(input: &str) -> String {
    let mut slug = String::new();
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
        } else if (ch.is_whitespace() || ch == '-' || ch == '_') && !slug.ends_with('_') {
            slug.push('_');
        }
    }

    if slug.is_empty() {
        "feature".to_string()
    } else {
        slug.trim_matches('_').to_string()
    }
}
