//! Action handlers for spec validation loop

use anyhow::Result;
use std::fs;
use std::io::Write;
use std::path::Path;

use crate::services::generator::refine_spec_from_idea;
use crate::services::scaffold::scaffold_with_spec_text;

use crate::tui::prompts::{confirm, edit_in_editor, print_error, print_info, print_success};
use crate::validation::print_diff;

/// Handle accept action - scaffold if confirmed
pub fn handle_accept(output_dir: &Path, spec_text: &str, is_valid: bool) -> Result<bool> {
    if !is_valid {
        let proceed = confirm("Spec has errors. Scaffold anyway?", false)?;
        if !proceed {
            return Ok(false);
        }
    }

    scaffold_with_spec_text(output_dir, spec_text)?;
    print_success("Project scaffolded successfully!");

    // Config was already done before generation, just show next steps
    println!("\n─── Next Steps ───");
    println!("  → Run opencode-forger vibe to start the autonomous coding loop");
    println!("  → Run opencode-forger --config to modify settings");
    println!();

    Ok(true)
}

/// Handle edit action - open in editor
pub fn handle_edit(spec_text: &mut String) -> Result<()> {
    print_info("Opening editor...");

    if let Some(edited) = edit_in_editor(spec_text)? {
        let old_spec = spec_text.clone();
        *spec_text = edited;
        print_diff(&old_spec, spec_text);
        print_success("Spec updated.");
    } else {
        println!("No changes.");
    }
    Ok(())
}

/// Handle save action - write to file
pub fn handle_save(output_dir: &Path, spec_text: &str) -> Result<()> {
    let spec_path = output_dir.join("app_spec.md");
    fs::write(&spec_path, spec_text)?;
    print_success(&format!("Saved to: {}", spec_path.display()));

    println!("\n─── Next Steps ───");
    println!("  → Run opencode-forger --config to configure settings");
    println!("  → Run opencode-forger vibe to start the autonomous coding loop");
    println!();

    Ok(())
}

/// Handle refine action - AI refinement with instructions
pub fn handle_refine(spec_text: &mut String, config: &crate::config::Config) -> Result<()> {
    let model = Some(config.models.default.as_str());
    println!("\n─── Refine Specification ───");
    display_spec_with_line_numbers(spec_text);

    println!("\nTIP: Reference line numbers or section names in your instructions");

    print!("Refinement instructions: ");
    let _ = std::io::stdout().flush();

    use std::io::BufRead;
    let stdin = std::io::stdin();
    let refinement = stdin.lock().lines().next().transpose()?.unwrap_or_default();

    if refinement.trim().is_empty() {
        println!("No instructions provided.");
        return Ok(());
    }

    println!("\n─────────────────────────────────────────────");

    let old_spec = spec_text.clone();

    match refine_spec_from_idea(spec_text, &refinement, model, config, |msg| {
        print!("{}", msg);
        let _ = std::io::stdout().flush();
    }) {
        Ok(refined) => {
            *spec_text = refined;
            println!("\nChanges made:");
            print_diff(&old_spec, spec_text);
            print_success("Specification refined.");
        }
        Err(e) => {
            print_error(&format!("Refinement failed: {}", e));
        }
    }
    Ok(())
}

fn display_spec_with_line_numbers(spec_text: &str) {
    println!("\nCurrent specification (with line numbers):");
    println!("{}", "─".repeat(60));

    for (i, line) in spec_text.lines().enumerate() {
        let line_num = i + 1;
        if line_num <= 50 {
            println!("{:4} │ {}", line_num, line);
        }
    }
    let total_lines = spec_text.lines().count();
    if total_lines > 50 {
        println!("     │ ... ({} more lines)", total_lines - 50);
    }
    println!("{}", "─".repeat(60));
}
