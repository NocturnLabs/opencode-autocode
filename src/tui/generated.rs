//! Generated mode - AI creates spec from idea

use anyhow::Result;
use console::style;
use crossterm::terminal::disable_raw_mode;
use dialoguer::{FuzzySelect, Input};
use std::io::Write;
use std::path::Path;

use crate::config_tui::fetch_available_models;
use crate::generator::generate_spec_from_idea;

use super::manual::run_manual_mode;
use super::validation::run_validation_loop;

/// Run AI-generated spec mode
pub fn run_generated_mode(
    output_dir: &Path,
    initial_model: Option<&str>,
    use_subagents: bool,
) -> Result<()> {
    println!("\n{}", style("─── AI Spec Generation ───").yellow().bold());
    println!(
        "{}",
        style("Describe your project and AI will create a comprehensive spec.").dim()
    );

    let model_owned = prompt_for_model(initial_model)?;
    let model = model_owned.as_deref();

    // Ensure terminal is in proper state after FuzzySelect
    // FuzzySelect uses raw mode which can leave artifacts
    let _ = disable_raw_mode();
    print!("\x1B[2K\r");
    let _ = std::io::stdout().flush();

    let idea = prompt_for_idea()?;
    if idea.is_empty() {
        return Ok(());
    }

    let mut spec_text = match generate_initial_spec(&idea, model, use_subagents) {
        Ok(spec) => spec,
        Err(e) => return handle_generation_error(e, output_dir),
    };

    run_validation_loop(output_dir, &mut spec_text, model_owned)
}

fn prompt_for_model(initial: Option<&str>) -> Result<Option<String>> {
    let models = match fetch_available_models() {
        Ok(m) => m,
        Err(_) => {
            println!(
                "{}",
                style("Could not fetch models, enter manually").yellow()
            );
            let input: String = Input::new()
                .with_prompt("Model (leave empty for default)")
                .default(initial.unwrap_or("").to_string())
                .allow_empty(true)
                .interact_text()?;
            return Ok(if input.trim().is_empty() {
                None
            } else {
                Some(input.trim().to_string())
            });
        }
    };

    println!(
        "{}",
        style(format!("Found {} available models", models.len())).dim()
    );

    let mut options = vec!["(Use default model)".to_string()];
    options.extend(models);
    options.push("(Enter custom model)".to_string());

    let default_idx = if let Some(init) = initial {
        options.iter().position(|m| m == init).unwrap_or(0)
    } else {
        0
    };

    let idx = FuzzySelect::new()
        .with_prompt("Select model (type to filter)")
        .items(&options)
        .default(default_idx)
        .max_length(5)
        .interact()?;

    if idx == 0 {
        Ok(None)
    } else if idx == options.len() - 1 {
        let custom: String = Input::new()
            .with_prompt("Enter model (provider/model)")
            .default(initial.unwrap_or("").to_string())
            .interact_text()?;
        Ok(Some(custom))
    } else {
        Ok(Some(options[idx].clone()))
    }
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
