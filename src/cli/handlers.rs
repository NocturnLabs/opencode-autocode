//! CLI Command Handlers
//!
//! This module contains the implementation logic for CLI subcommands,
//! extracted from main.rs to improve modularity and testability.

use anyhow::Result;
use std::path::PathBuf;

use crate::config_tui;
use crate::services::scaffold;
use crate::theming::{accent, error, highlight, muted, primary, symbols};
use crate::tui;
use crate::updater;

use super::commands::{db, example, init, reset, templates, vibe};
use super::{Cli, Commands, Mode};

/// Main entry point for handling CLI commands.
///
/// Parses the CLI arguments and dispatches to the appropriate handler.
pub fn run(cli: Cli) -> Result<()> {
    let output_dir = cli.output.clone().unwrap_or_else(|| PathBuf::from("."));

    // Handle subcommands first
    if let Some(command) = &cli.command {
        return match command {
            Commands::Vibe {
                limit,
                config_file,
                developer,
                single_model,
                parallel,
                feature_id,
            } => vibe::handle_vibe(
                *limit,
                config_file.as_deref(),
                *developer,
                *single_model,
                *parallel,
                *feature_id,
            ),
            Commands::Enhance {
                limit,
                config_file,
                developer,
                single_model,
            } => crate::autonomous::run(
                *limit,
                config_file.as_deref(),
                *developer,
                *single_model,
                true,
                None,
            ),
            Commands::Init {
                default,
                spec,
                no_subagents,
            } => init::handle_init(
                &output_dir,
                *default,
                spec.as_deref(),
                *no_subagents,
                cli.dry_run,
            ),
            Commands::Templates { action } => templates::handle_templates(action, &output_dir),
            Commands::Db { action } => db::handle_db(action),
            Commands::Example { topic } => example::handle_example(topic),
            Commands::Update => match updater::update() {
                Ok(_) => Ok(()),
                Err(e) => {
                    eprintln!(
                        "{} {}",
                        error(symbols::ERROR),
                        error(format!("Failed to update: {}", e))
                    );
                    std::process::exit(1);
                }
            },
            Commands::Web { port, open } => crate::web::run_server(*port, *open),
        };
    }

    // Handle flag-based modes
    match cli.mode()? {
        Mode::Config => config_tui::run_config_tui(None).map(|_| ()),
        Mode::Default => {
            if cli.dry_run {
                println!(
                    "{} {}",
                    accent(symbols::INFO),
                    muted("Dry run mode - no files will be created")
                );
                scaffold::preview_scaffold(&output_dir);
                return Ok(());
            }
            init::print_next_steps(&output_dir);
            scaffold::scaffold_default(&output_dir)?;
            Ok(())
        }
        Mode::Custom(spec_path) => {
            if cli.dry_run {
                println!(
                    "{} {}",
                    accent(symbols::INFO),
                    muted("Dry run mode - no files will be created")
                );
                scaffold::preview_scaffold(&output_dir);
                return Ok(());
            }
            println!(
                "{} {} {}",
                accent(symbols::CHEVRON),
                highlight("Scaffolding with custom spec:"),
                primary(spec_path.display())
            );
            scaffold::scaffold_custom(&output_dir, &spec_path)?;
            init::print_next_steps(&output_dir);
            Ok(())
        }
        Mode::Reset => reset::handle_reset(&output_dir),
        Mode::Interactive => {
            // Check for updates
            if let Ok(Some(new_version)) = updater::check_for_update() {
                println!(
                    "\n{} {} {} {}\n",
                    accent(symbols::SPARKLE),
                    highlight("New version available:"),
                    primary(new_version),
                    muted("(Run 'opencode-forger update' to upgrade)")
                );
            }

            if cli.dry_run {
                println!(
                    "{} {}",
                    accent(symbols::INFO),
                    muted("Dry run mode - no files will be created")
                );
                scaffold::preview_scaffold(&output_dir);
                return Ok(());
            }
            tui::run_interactive(&output_dir, !cli.no_subagents)?;
            Ok(())
        }
    }
}
