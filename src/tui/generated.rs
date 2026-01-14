//! Generated mode - AI creates spec from idea

use anyhow::Result;
use std::io::Write;
use std::path::Path;

use crate::services::generator::generate_spec_from_idea;
use crate::theming::{accent, highlight, muted, primary, symbols};

use crate::tui::prompts::{confirm, multiline_input, print_error, print_info};

use super::manual::run_manual_mode;
use super::validation::run_validation_loop;

/// Run AI-generated spec mode
pub fn run_generated_mode(
    output_dir: &Path,
    config: &crate::config::Config,
    use_subagents: bool,
) -> Result<()> {
    println!(
        "\n{} {}",
        accent(symbols::CHEVRON),
        highlight("AI Spec Generation")
    );
    print_info("Describe your project and AI will create a comprehensive spec.");
    println!(
        "{} {}",
        muted("Using model:"),
        primary(&config.models.default)
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

    let mut spec_text = match generate_initial_spec(
        &idea,
        testing_pref.as_deref(),
        model,
        use_subagents,
        config,
        config.ui.show_progress,
    ) {
        Ok(spec) => spec,
        Err(e) => return handle_generation_error(e, output_dir, config),
    };

    run_validation_loop(output_dir, &mut spec_text, config)
}

fn prompt_for_idea() -> Result<String> {
    multiline_input("Describe your project idea")
}

fn prompt_for_testing_preference() -> Result<Option<String>> {
    use std::io::{self, BufRead};

    println!();
    print!("Testing framework preference (optional, press Enter to let AI decide): ");
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
    config: &crate::config::Config,
    show_progress: bool,
) -> Result<String> {
    print!("\x1B[2K\r");
    let _ = std::io::stdout().flush();

    if show_progress {
        println!(
            "\n{}",
            muted("─────────────────────────────────────────────")
        );
    }

    generate_spec_from_idea(idea, testing_pref, model, use_subagents, config, |msg| {
        if show_progress {
            print!("{}", msg);
            let _ = std::io::stdout().flush();
        }
    })
}

fn handle_generation_error(
    e: anyhow::Error,
    output_dir: &Path,
    config: &crate::config::Config,
) -> Result<()> {
    print_error(&format!("{}", e));

    if confirm("Switch to manual mode?", true)? {
        run_manual_mode(output_dir, config)
    } else {
        Ok(())
    }
}
