use crate::services::scaffold;
use anyhow::Result;

pub fn handle_reset(output_dir: &std::path::Path) -> Result<()> {
    let spec_path = output_dir.join(".forger/app_spec.md");
    if !spec_path.exists() {
        anyhow::bail!(
            "Cannot reset: .forger/app_spec.md not found in {}",
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
