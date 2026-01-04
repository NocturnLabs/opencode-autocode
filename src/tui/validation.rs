//! Validation loop for spec review and refinement

use anyhow::Result;
use console::style;
use dialoguer::Select;
use std::path::Path;

use crate::validation::validate_spec;

/// Actions available after spec generation
pub enum SpecAction {
    Accept,
    Edit,
    SaveToFile,
    Refine,
    Regenerate,
    Cancel,
}

/// Run the validation and action loop for a generated spec
pub fn run_validation_loop(
    output_dir: &Path,
    spec_text: &mut String,
    model_owned: Option<String>,
    min_features: usize,
    min_endpoints: usize,
    spec_preview_lines: u32,
) -> Result<()> {
    loop {
        let validation =
            validate_and_preview(spec_text, min_features, min_endpoints, spec_preview_lines)?;
        let action = prompt_for_action(validation.is_valid)?;

        match action {
            SpecAction::Accept => {
                if super::actions::handle_accept(output_dir, spec_text, validation.is_valid)? {
                    break;
                }
            }
            SpecAction::Edit => super::actions::handle_edit(spec_text)?,
            SpecAction::SaveToFile => {
                super::actions::handle_save(output_dir, spec_text)?;
                break;
            }
            SpecAction::Refine => super::actions::handle_refine(spec_text, model_owned.as_deref())?,
            SpecAction::Regenerate => {
                // Load config and regenerate with subagents enabled
                let config = crate::config::Config::load(Some(output_dir)).unwrap_or_default();
                return super::generated::run_generated_mode(output_dir, &config, true);
            }
            SpecAction::Cancel => {
                println!("{}", style("Cancelled.").red());
                break;
            }
        }
    }
    Ok(())
}

fn validate_and_preview(
    spec_text: &str,
    min_features: usize,
    min_endpoints: usize,
    spec_preview_lines: u32,
) -> Result<crate::validation::ValidationResult> {
    println!(
        "\n{}",
        style("─── Validating Specification ───").cyan().bold()
    );

    let validation = validate_spec(spec_text, min_features, min_endpoints)?;
    validation.print();

    if !validation.is_valid {
        println!("\n{}", style("The spec has validation errors.").red());
    }

    display_spec_preview(spec_text, spec_preview_lines);
    Ok(validation)
}

fn display_spec_preview(spec_text: &str, max_lines: u32) {
    println!(
        "\n{}",
        style("═══════════════════════════════════════════════════").green()
    );
    println!(
        "{}",
        style("  Generated Specification Preview").green().bold()
    );
    println!(
        "{}\n",
        style("═══════════════════════════════════════════════════").green()
    );

    let max = max_lines as usize;
    for line in spec_text.lines().take(max) {
        println!("  {}", style(line).dim());
    }
    let total_lines = spec_text.lines().count();
    if total_lines > max {
        println!(
            "  {}",
            style(format!("... ({} more lines)", total_lines - max)).dim()
        );
    }
}

fn prompt_for_action(is_valid: bool) -> Result<SpecAction> {
    let options = if is_valid {
        vec![
            "✅ Accept and scaffold",
            "✏️  Edit manually",
            "📄 Save to file",
            "🔧 Refine with instructions",
            "🔄 Regenerate",
            "❌ Cancel",
        ]
    } else {
        vec![
            "⚠️  Accept anyway (has errors)",
            "✏️  Edit manually",
            "📄 Save to file",
            "🔧 Refine with instructions",
            "🔄 Regenerate",
            "❌ Cancel",
        ]
    };

    let idx = Select::new()
        .with_prompt("What would you like to do?")
        .items(&options)
        .default(if is_valid { 0 } else { 1 })
        .interact()?;

    Ok(match idx {
        0 => SpecAction::Accept,
        1 => SpecAction::Edit,
        2 => SpecAction::SaveToFile,
        3 => SpecAction::Refine,
        4 => SpecAction::Regenerate,
        _ => SpecAction::Cancel,
    })
}
