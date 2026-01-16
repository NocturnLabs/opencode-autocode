//! CLI Command Handlers
//!
//! This module contains the implementation logic for CLI subcommands,
//! extracted from main.rs to improve modularity and testability.

use anyhow::Result;
use std::env;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::config_tui;
use crate::ipc::{self, IpcServer, ModeInfo};
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
    let output_dir = cli.output.clone().unwrap_or_else(|| {
        let config = Config::load(None).unwrap_or_default();
        if config.scaffolding.output_dir.trim().is_empty() {
            PathBuf::from(".")
        } else {
            PathBuf::from(config.scaffolding.output_dir)
        }
    });
    let output_dir = resolve_output_dir(output_dir);

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

            // Try Go TUI first, fall back to legacy Rust TUI
            if IpcServer::is_available() {
                run_interactive_ipc(&output_dir, !cli.no_subagents)
            } else {
                // Fall back to legacy Rust TUI
                tui::run_interactive(&output_dir, !cli.no_subagents)?;
                Ok(())
            }
        }
    }
}

/// Run the interactive mode using the Go TUI via IPC.
///
/// This function spawns the Go Bubble Tea client and communicates with it
/// via JSON-RPC over stdin/stdout.
fn resolve_output_dir(output_dir: PathBuf) -> PathBuf {
    if output_dir.is_absolute() {
        return output_dir;
    }

    match env::current_dir() {
        Ok(current_dir) => current_dir.join(output_dir),
        Err(_) => output_dir,
    }
}

fn run_interactive_ipc(output_dir: &Path, use_subagents: bool) -> Result<()> {
    use crate::tui::fullscreen::types::InteractiveMode;

    // Spawn the Go TUI
    let server = IpcServer::spawn()?;

    // Get version from cargo
    let version = env!("CARGO_PKG_VERSION");
    let work_dir = output_dir
        .canonicalize()
        .unwrap_or_else(|_| output_dir.to_path_buf())
        .display()
        .to_string();

    // Send engine ready event
    server.send_engine_ready(version, &work_dir)?;

    // Check for existing config
    let config_path = output_dir.join(".forger/config.toml");
    let has_existing_config = config_path.exists();
    let config_path_str = config_path.display().to_string();
    server.send_config_loaded(
        has_existing_config,
        has_existing_config.then_some(config_path_str.as_str()),
    )?;

    // Send available modes
    let modes = InteractiveMode::all();
    let mode_infos: Vec<ModeInfo> = modes
        .iter()
        .map(|m: &InteractiveMode| ModeInfo {
            id: m.id().to_string(),
            label: m.label().to_string(),
            description: m.description().to_string(),
        })
        .collect();
    server.send_mode_list(mode_infos)?;

    // Wait for mode selection
    #[allow(unused_assignments)]
    let mut selected_mode: Option<InteractiveMode> = None;
    #[allow(unused_assignments)]
    let mut should_configure = false;

    loop {
        let msg = server.recv_command()?;

        match msg.name.as_str() {
            ipc::commands::CONFIRM => {
                if let Some(payload) = msg.payload {
                    if let Ok(confirm) = serde_json::from_value::<ipc::ConfirmPayload>(payload) {
                        should_configure = confirm.confirmed;
                    }
                }
            }
            ipc::commands::SELECT_MODE => {
                if let Some(payload) = msg.payload {
                    if let Ok(select) = serde_json::from_value::<ipc::SelectModePayload>(payload) {
                        selected_mode = modes
                            .iter()
                            .find(|m: &&InteractiveMode| m.id() == select.mode_id)
                            .copied();
                        if selected_mode.is_some() {
                            break;
                        }
                    }
                }
            }
            ipc::commands::CANCEL => {
                server.send_finished(false, Some("Cancelled by user"))?;
                server.shutdown()?;
                println!("Cancelled.");
                return Ok(());
            }
            _ => {}
        }
    }

    // Handle configuration if requested
    let config = if should_configure {
        server.send_log("info", "Opening configuration editor...")?;
        // Note: config_tui is still Rust-based for now
        // In the future, this could also be handled via IPC
        config_tui::run_config_tui(Some(output_dir))?
    } else {
        Config::load(Some(output_dir)).unwrap_or_default()
    };

    // Execute the selected mode
    match selected_mode {
        Some(InteractiveMode::Generated) => {
            server.send_progress(
                "scaffolding",
                0,
                1,
                Some("Generating project specification..."),
            )?;

            // Run the generated mode (still uses Rust TUI for AI interactions)
            // In the future, we can send prompts via IPC
            let result =
                crate::tui::generated::run_generated_mode(output_dir, &config, use_subagents);

            match &result {
                Ok(()) => {
                    server.send_progress("scaffolding", 1, 1, Some("Complete!"))?;
                    server.send_finished(true, Some("Project scaffolded successfully!"))?;
                }
                Err(e) => {
                    server.send_error(&e.to_string(), true)?;
                    server.send_finished(false, Some(&e.to_string()))?;
                }
            }
            server.shutdown()?;
            result
        }
        Some(InteractiveMode::Manual) => {
            server.send_progress("scaffolding", 0, 1, Some("Starting manual mode..."))?;
            let result = crate::tui::manual::run_manual_mode(output_dir, &config);
            match &result {
                Ok(()) => {
                    server.send_finished(true, Some("Project scaffolded successfully!"))?;
                }
                Err(e) => {
                    server.send_finished(false, Some(&e.to_string()))?;
                }
            }
            server.shutdown()?;
            result
        }
        Some(InteractiveMode::FromSpecFile) => {
            server.send_progress("scaffolding", 0, 1, Some("Loading spec file..."))?;

            // Prompt for spec file path
            let default_spec = if config.paths.app_spec_file.trim().is_empty() {
                ".forger/app_spec.md".to_string()
            } else {
                config.paths.app_spec_file.clone()
            };

            let selection = server.prompt_select(
                "spec_file",
                "Enter path to spec file:",
                vec![default_spec.clone()],
            )?;

            let spec_path = std::path::PathBuf::from(&selection.value);
            if !spec_path.exists() {
                server.send_error(
                    &format!("Spec file not found: {}", spec_path.display()),
                    true,
                )?;
                server.send_finished(false, Some("Spec file not found"))?;
                server.shutdown()?;
                return Ok(());
            }

            scaffold::scaffold_custom(output_dir, &spec_path)?;
            server.send_finished(true, Some("Project scaffolded from spec file!"))?;
            server.shutdown()?;
            Ok(())
        }
        Some(InteractiveMode::Default) => {
            server.send_progress(
                "scaffolding",
                0,
                1,
                Some("Scaffolding with default spec..."),
            )?;
            scaffold::scaffold_default(output_dir)?;
            server.send_progress("scaffolding", 1, 1, Some("Complete!"))?;
            server.send_finished(true, Some("Project scaffolded with default spec!"))?;
            server.shutdown()?;
            Ok(())
        }
        None => {
            server.send_finished(false, Some("No mode selected"))?;
            server.shutdown()?;
            Ok(())
        }
    }
}
