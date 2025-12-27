//! Generated mode - AI creates spec from idea

use anyhow::Result;
use console::style;
use std::io::Write;
use std::path::Path;

use crate::generator::generate_spec_from_idea;

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

    let mut spec_text = match generate_initial_spec(&idea, model, use_subagents) {
        Ok(spec) => spec,
        Err(e) => return handle_generation_error(e, output_dir),
    };

    run_validation_loop(
        output_dir,
        &mut spec_text,
        Some(config.models.default.clone()),
    )
}

fn prompt_for_idea() -> Result<String> {
    use std::io::{self, BufRead};

    print!("{}: ", style("Describe your project idea").green());
    let _ = std::io::stdout().flush();

    let stdin = io::stdin();
    let idea = stdin.lock().lines().next().transpose()?.unwrap_or_default();

    if idea.trim().is_empty() {
        println!("{}", style("No idea provided.").red());
        return Ok(String::new());
    }

    Ok(idea)
}

fn generate_initial_spec(idea: &str, model: Option<&str>, use_subagents: bool) -> Result<String> {
    print!("\x1B[2K\r");
    let _ = std::io::stdout().flush();

    println!(
        "\n{}",
        style("─────────────────────────────────────────────").dim()
    );

    generate_spec_from_idea(idea, model, use_subagents, |msg| {
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
