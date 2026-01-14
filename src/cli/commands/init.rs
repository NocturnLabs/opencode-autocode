use crate::services::scaffold;
use crate::tui;
use anyhow::Result;

/// Handles the `init` subcommand for project initialization.
///
/// This function scaffolds a new project based on the provided options:
/// - Dry-run mode previews scaffolding without creating files
/// - Default mode uses the embedded app spec template
/// - Custom spec mode uses a user-provided spec file
/// - Interactive mode launches the TUI for guided spec building
///
/// # Arguments
///
/// * `output_dir` - Target directory for scaffolded files.
/// * `default` - Whether to use the default embedded spec template.
/// * `spec` - Optional path to a custom spec file.
/// * `no_subagents` - Whether to disable parallel subagent spec generation.
/// * `dry_run` - Whether to preview scaffolding without creating files.
///
/// # Returns
///
/// Result indicating success or containing an error from scaffolding/TUI.
pub fn handle_init(
    output_dir: &std::path::Path,
    default: bool,
    spec: Option<&std::path::Path>,
    no_subagents: bool,
    dry_run: bool,
) -> Result<()> {
    if dry_run {
        println!("🔍 Dry run mode - no files will be created");
        scaffold::preview_scaffold(output_dir);
        return Ok(());
    }

    if default {
        println!("🚀 Scaffolding with default app spec...");
        scaffold::scaffold_default(output_dir)?;
        print_next_steps(output_dir);
        Ok(())
    } else if let Some(spec_path) = spec {
        if !spec_path.exists() {
            anyhow::bail!("Spec file not found: {}", spec_path.display());
        }
        println!("📄 Scaffolding with custom spec: {}", spec_path.display());
        scaffold::scaffold_custom(output_dir, spec_path)?;
        print_next_steps(output_dir);
        Ok(())
    } else {
        tui::run_interactive(output_dir, !no_subagents)?;
        Ok(())
    }
}

/// Prints the next steps after scaffolding is complete.
///
/// # Arguments
///
/// * `output_dir` - The directory where the project was scaffolded.
pub fn print_next_steps(output_dir: &std::path::Path) {
    println!("\n✅ Scaffolding complete!");
    println!("   Output directory: {}", output_dir.display());
    println!("\n📋 Next steps:");
    println!("   1. cd {}", output_dir.display());
    println!("   2. opencode-forger --config  # Configure settings");
    println!("   3. opencode-forger example   # See agent-centric examples and guides");
    println!("   4. opencode-forger vibe      # Start autonomous loop");
}
