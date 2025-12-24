//! OpenCode Autocode - Autonomous Coding for OpenCode
//!
//! A CLI tool that scaffolds autonomous coding projects and runs
//! the vibe loop to implement features automatically.

#![deny(warnings)]

mod autonomous;
mod cli;
mod config;
mod config_tui;
mod generator;
mod regression;
mod scaffold;
mod spec;
mod templates;
mod tui;
mod validation;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands, Mode, TemplateAction};
use std::path::PathBuf;

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Determine output directory
    let output_dir = cli.output.clone().unwrap_or_else(|| PathBuf::from("."));

    // Handle subcommands first (vibe is the main one)
    if let Some(command) = &cli.command {
        return match command {
            Commands::Vibe {
                limit,
                config_file,
                developer,
            } => autonomous::run(*limit, config_file.as_deref(), *developer),
            Commands::Templates { action } => match action {
                TemplateAction::List => {
                    templates::list_templates();
                    Ok(())
                }
                TemplateAction::Use { name } => templates::use_template(name, &output_dir),
            },
        };
    }

    // Handle flag-based modes
    match cli.mode()? {
        Mode::Config => config_tui::run_config_tui(),
        Mode::RegressionCheck => {
            let feature_path = cli
                .feature_list
                .clone()
                .unwrap_or_else(|| PathBuf::from("feature_list.json"));

            if !feature_path.exists() {
                anyhow::bail!("Feature list not found: {}", feature_path.display());
            }

            println!(
                "🔍 Running regression check on {}...",
                feature_path.display()
            );

            let summary = regression::run_regression_check(&feature_path, None, cli.verbose)?;
            regression::report_results(&summary);

            if summary.automated_failed > 0 {
                std::process::exit(1);
            }
            Ok(())
        }
        Mode::Default => {
            if cli.dry_run {
                println!("🔍 Dry run mode - no files will be created");
                scaffold::preview_scaffold(&output_dir);
                return Ok(());
            }
            println!("🚀 Scaffolding with default app spec...");
            scaffold::scaffold_default(&output_dir)?;
            print_next_steps(&output_dir);
            Ok(())
        }
        Mode::Custom(spec_path) => {
            if cli.dry_run {
                println!("🔍 Dry run mode - no files will be created");
                scaffold::preview_scaffold(&output_dir);
                return Ok(());
            }
            println!("📄 Scaffolding with custom spec: {}", spec_path.display());
            scaffold::scaffold_custom(&output_dir, &spec_path)?;
            print_next_steps(&output_dir);
            Ok(())
        }
        Mode::Interactive => {
            if cli.dry_run {
                println!("🔍 Dry run mode - no files will be created");
                scaffold::preview_scaffold(&output_dir);
                return Ok(());
            }
            tui::run_interactive(&output_dir)?;
            Ok(())
        }
    }
}

fn print_next_steps(output_dir: &std::path::Path) {
    println!("\n✅ Scaffolding complete!");
    println!("   Output directory: {}", output_dir.display());
    println!("\n📋 Next steps:");
    println!("   1. cd {}", output_dir.display());
    println!("   2. opencode-autocode --config  # Configure settings");
    println!("   3. opencode-autocode vibe      # Start autonomous loop");
}
