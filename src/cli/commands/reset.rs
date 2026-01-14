use crate::services::scaffold;
use anyhow::Result;

use crate::config::Config;
use std::path::{Path, PathBuf};

pub fn handle_reset(output_dir: &std::path::Path) -> Result<()> {
    let config = Config::load(Some(output_dir)).unwrap_or_default();
    let spec_path = resolve_spec_path(output_dir, &config.paths.app_spec_file);
    if !spec_path.exists() {
        anyhow::bail!(
            "Cannot reset: {} not found in {}",
            spec_path.display(),
            output_dir.display()
        );
    }

    println!("🔄 Resetting project (preserving database)...");

    // Clean up ONLY temporary/signal files - PRESERVE the .db!
    let files_to_remove = [
        output_dir.join(".opencode-signal"),
        output_dir.join(".opencode-stop"),
    ];
    for path in &files_to_remove {
        if path.exists() {
            std::fs::remove_file(path)?;
            println!("   🗑️  Removed {}", path.display());
        }
    }

    // Remove .opencode/command directory to get fresh templates
    let command_dir = output_dir.join(".opencode/command");
    if command_dir.exists() {
        std::fs::remove_dir_all(&command_dir)?;
        println!("   🗑️  Removed {}", command_dir.display());
    }

    // Re-scaffold with existing spec
    println!("   📄 Re-scaffolding with existing spec...");
    scaffold::scaffold_custom(output_dir, &spec_path)?;

    println!("\n✅ Reset complete! Project is ready for a fresh run.");
    println!("   Run 'opencode-forger vibe' to start the autonomous loop.");
    Ok(())
}

/// @param output_dir Base output directory.
/// @param spec_path Configured spec file path.
/// @returns Resolved spec path for the reset command.
fn resolve_spec_path(output_dir: &Path, spec_path: &str) -> PathBuf {
    let trimmed = spec_path.trim();
    if trimmed.is_empty() {
        return output_dir.join(".forger/app_spec.md");
    }

    let path = Path::new(trimmed);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        output_dir.join(path)
    }
}
