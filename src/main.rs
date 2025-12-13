//! OpenCode Autonomous Coding Plugin Scaffolder
//!
//! A CLI tool that scaffolds an autonomous coding plugin for OpenCode,
//! enhanced with custom MCP integrations.

mod cli;
mod config;
mod editor;
mod generator;
mod regression;
mod scaffold;
mod spec;
mod templates;
mod tui;
mod validation;
mod verbalized_sampling;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands, Mode, TemplateAction};
use std::path::PathBuf;

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Determine output directory
    let output_dir = cli.output.clone().unwrap_or_else(|| PathBuf::from("."));

    // Handle subcommands first
    if let Some(command) = &cli.command {
        return match command {
            Commands::Templates { action } => match action {
                TemplateAction::List => {
                    templates::list_templates();
                    Ok(())
                }
                TemplateAction::Use { name } => templates::use_template(name, &output_dir),
            },
            Commands::Edit => editor::run_editor(&output_dir),
            Commands::RegressionCheck {
                feature_list,
                category,
                verbose,
            } => {
                let feature_path = feature_list
                    .clone()
                    .unwrap_or_else(|| PathBuf::from("feature_list.json"));

                if !feature_path.exists() {
                    anyhow::bail!("Feature list not found: {}", feature_path.display());
                }

                println!(
                    "🔍 Running regression check on {}...",
                    feature_path.display()
                );

                let summary =
                    regression::run_regression_check(&feature_path, category.as_deref(), *verbose)?;

                regression::report_results(&summary);

                if summary.automated_failed > 0 {
                    std::process::exit(1);
                }
                Ok(())
            }
        };
    }

    // Handle dry-run mode
    if cli.dry_run {
        println!("🔍 Dry run mode - no files will be created");
        scaffold::preview_scaffold(&output_dir);
        return Ok(());
    }

    // Execute based on mode
    match cli.mode()? {
        Mode::Default => {
            println!("🚀 Scaffolding with default app spec...");
            scaffold::scaffold_default(&output_dir)?;
        }
        Mode::Custom(spec_path) => {
            println!("📄 Scaffolding with custom spec: {}", spec_path.display());
            scaffold::scaffold_custom(&output_dir, &spec_path)?;
        }
        Mode::Interactive => {
            tui::run_interactive(&output_dir)?;
            return Ok(()); // TUI handles its own output
        }
    }

    println!("\n✅ Scaffolding complete!");
    println!("   Output directory: {}", output_dir.display());
    println!("\n📋 Next steps:");
    println!("   1. cd {}", output_dir.display());
    println!("   2. Edit app_spec.md to define your project");
    println!("   3. Run: opencode /auto-init");

    Ok(())
}
