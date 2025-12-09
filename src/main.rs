//! OpenCode Autonomous Coding Plugin Scaffolder
//!
//! A CLI tool that scaffolds an autonomous coding plugin for OpenCode,
//! enhanced with custom MCP integrations.

mod cli;
mod scaffold;
mod spec;
mod tui;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Mode};
use std::path::PathBuf;

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Determine output directory
    let output_dir = cli.output.clone().unwrap_or_else(|| PathBuf::from("."));

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
            println!("🎨 Launching interactive TUI...");
            tui::run_interactive(&output_dir)?;
        }
    }

    println!("\n✅ Scaffolding complete!");
    println!("   Output directory: {}", output_dir.display());
    println!("\n📋 Next steps:");
    println!("   1. cd {}", output_dir.display());
    println!("   2. Edit app_spec.txt to define your project");
    println!("   3. Run: opencode /auto-init");

    Ok(())
}
