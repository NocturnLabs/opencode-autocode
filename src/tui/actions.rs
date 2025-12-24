//! Action handlers for spec validation loop

use anyhow::Result;
use console::style;
use dialoguer::{Confirm, Editor, Input};
use std::fs;
use std::io::Write;
use std::path::Path;

use crate::config_tui::run_config_tui;
use crate::generator::refine_spec_from_idea;
use crate::scaffold::scaffold_with_spec_text;
use crate::validation::print_diff;

/// Handle accept action - scaffold if confirmed
pub fn handle_accept(output_dir: &Path, spec_text: &str, is_valid: bool) -> Result<bool> {
    if !is_valid {
        let confirm = Confirm::new()
            .with_prompt("Spec has errors. Scaffold anyway?")
            .default(false)
            .interact()?;
        if !confirm {
            return Ok(false);
        }
    }

    scaffold_with_spec_text(output_dir, spec_text)?;
    println!(
        "\n{}",
        style("✅ Project scaffolded successfully!").green().bold()
    );

    println!();
    if Confirm::new()
        .with_prompt("Would you like to configure project settings now?")
        .default(true)
        .interact()?
    {
        println!();
        run_config_tui()?;
    } else {
        println!(
            "\n{}",
            style("Run 'opencode-autocode --config' later to configure settings").dim()
        );
    }

    Ok(true)
}

/// Handle edit action - open in editor
pub fn handle_edit(spec_text: &mut String) -> Result<()> {
    println!("{}", style("Opening editor...").dim());

    if let Some(edited) = Editor::new().edit(spec_text)? {
        let old_spec = spec_text.clone();
        *spec_text = edited;
        print_diff(&old_spec, spec_text);
        println!("{}", style("Spec updated.").cyan());
    } else {
        println!("{}", style("No changes.").dim());
    }
    Ok(())
}

/// Handle save action - write to file
pub fn handle_save(output_dir: &Path, spec_text: &str) -> Result<()> {
    let spec_path = output_dir.join("app_spec.md");
    fs::write(&spec_path, spec_text)?;
    println!(
        "\n{} {}",
        style("📄 Saved to:").cyan(),
        style(spec_path.display()).green()
    );

    println!("\n{}", style("─── Next Steps ───").cyan().bold());
    println!(
        "  {} Run {} to configure settings",
        style("→").cyan(),
        style("opencode-autocode --config").green()
    );
    println!(
        "  {} Run {} to start the autonomous coding loop",
        style("→").cyan(),
        style("opencode-autocode vibe").green().bold()
    );
    println!();

    Ok(())
}

/// Handle refine action - AI refinement with instructions
pub fn handle_refine(spec_text: &mut String, model: Option<&str>) -> Result<()> {
    println!(
        "\n{}",
        style("─── Refine Specification ───").yellow().bold()
    );
    display_spec_with_line_numbers(spec_text);

    println!(
        "\n{}",
        style("TIP: Reference line numbers or section names in your instructions").dim()
    );

    let refinement: String = Input::new()
        .with_prompt("Refinement instructions")
        .interact_text()?;

    if refinement.trim().is_empty() {
        println!("{}", style("No instructions provided.").dim());
        return Ok(());
    }

    println!(
        "\n{}",
        style("─────────────────────────────────────────────").dim()
    );

    let old_spec = spec_text.clone();

    match refine_spec_from_idea(spec_text, &refinement, model, |msg| {
        print!("{}", msg);
        let _ = std::io::stdout().flush();
    }) {
        Ok(refined) => {
            *spec_text = refined;
            println!("\n{}", style("Changes made:").cyan().bold());
            print_diff(&old_spec, spec_text);
            println!("\n{}", style("Specification refined.").cyan());
        }
        Err(e) => {
            println!("\n{} {}", style("Refinement failed:").red().bold(), e);
        }
    }
    Ok(())
}

fn display_spec_with_line_numbers(spec_text: &str) {
    println!(
        "\n{}",
        style("Current specification (with line numbers):").cyan()
    );
    println!("{}", style("─".repeat(60)).dim());

    for (i, line) in spec_text.lines().enumerate() {
        let line_num = i + 1;
        if line_num <= 50 {
            println!("{:4} │ {}", style(line_num).dim(), line);
        }
    }
    let total_lines = spec_text.lines().count();
    if total_lines > 50 {
        println!(
            "     │ {}",
            style(format!("... ({} more lines)", total_lines - 50)).dim()
        );
    }
    println!("{}", style("─".repeat(60)).dim());
}
