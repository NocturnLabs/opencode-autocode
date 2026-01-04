//! Generated mode - AI creates spec from idea

use anyhow::Result;
use console::style;
use std::io::Write;
use std::path::Path;

use crate::generator::generate_spec_from_idea;

use super::inputs::read_multiline;
use super::manual::run_manual_mode;
use super::validation::run_validation_loop;

/// Run AI-generated spec mode
pub fn run_generated_mode(
    output_dir: &Path,
    config: &crate::config::Config,
    use_subagents: bool,
) -> Result<()> {
    println!("\n{}", style("─── AI Spec Generation ───").yellow().bold());
    println!(
        "{}",
        style("Describe your project and AI will create a comprehensive spec.").dim()
    );
    println!(
        "{}",
        style(format!("Using model: {}", config.models.default)).dim()
    );

    // Use model from config directly (config was set before generation)
    let model = Some(config.models.default.as_str());

    // Respect config's subagent setting (CLI flag can disable, but not enable if config disables)
    let use_subagents = use_subagents && config.generation.enable_subagents;

    let idea = prompt_for_idea()?;
    if idea.is_empty() {
        return Ok(());
    }

    let testing_pref = prompt_for_testing_preference()?;

    let mut spec_text =
        match generate_initial_spec(&idea, testing_pref.as_deref(), model, use_subagents) {
            Ok(spec) => spec,
            Err(e) => return handle_generation_error(e, output_dir),
        };

    let (min_f, min_e) = if config.generation.complexity == "minimal" {
        (
            config.generation.minimal_min_features as usize,
            config.generation.minimal_min_api_endpoints as usize,
        )
    } else {
        (
            config.generation.min_features as usize,
            config.generation.min_api_endpoints as usize,
        )
    };

    run_validation_loop(
        output_dir,
        &mut spec_text,
        Some(config.models.default.clone()),
        min_f,
        min_e,
    )
}

fn prompt_for_idea() -> Result<String> {
    read_multiline("Describe your project idea")
}

fn prompt_for_testing_preference() -> Result<Option<String>> {
    use std::io::{self, BufRead};

    println!();
    print!(
        "{}: ",
        style("Testing framework preference (optional, press Enter to let AI decide)").blue()
    );
    let _ = std::io::stdout().flush();

    let stdin = io::stdin();
    let pref = stdin.lock().lines().next().transpose()?.unwrap_or_default();

    if pref.trim().is_empty() {
        Ok(None)
    } else {
        Ok(Some(pref))
    }
}

fn generate_initial_spec(
    idea: &str,
    testing_pref: Option<&str>,
    model: Option<&str>,
    use_subagents: bool,
) -> Result<String> {
    print!("\x1B[2K\r");
    let _ = std::io::stdout().flush();

    println!(
        "\n{}",
        style("─────────────────────────────────────────────").dim()
    );

    generate_spec_from_idea(idea, testing_pref, model, use_subagents, |msg| {
        print!("{}", msg);
        let _ = std::io::stdout().flush();
    })
}

fn handle_generation_error(e: anyhow::Error, output_dir: &Path) -> Result<()> {
    use dialoguer::Confirm;

    println!("\n{} {}", style("Error:").red().bold(), e);

    if Confirm::new()
        .with_prompt("Switch to manual mode?")
        .default(true)
        .interact()?
    {
        run_manual_mode(output_dir)
    } else {
        Ok(())
    }
}
