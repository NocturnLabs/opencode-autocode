//! Validation loop for spec review and refinement

use anyhow::Result;
use std::path::Path;

use super::fullscreen::run_fullscreen_spec_review;
use crate::validation::validate_spec_with_config;

/// Actions available after spec generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    config: &crate::config::Config,
) -> Result<()> {
    loop {
        let validation = validate_spec_with_config(spec_text, config)?;

        // Run fullscreen TUI for spec review
        let action = run_fullscreen_spec_review(
            spec_text,
            validation.clone(),
            config.ui.spec_preview_lines as usize,
        )?;

        match action {
            SpecAction::Accept => {
                if super::actions::handle_accept(output_dir, spec_text, validation.is_valid)? {
                    break;
                }
            }
            SpecAction::Edit => super::actions::handle_edit(spec_text)?,
            SpecAction::SaveToFile => {
                super::actions::handle_save(output_dir, spec_text, config)?;
                break;
            }
            SpecAction::Refine => super::actions::handle_refine(spec_text, config)?,
            SpecAction::Regenerate => {
                return super::generated::run_generated_mode(output_dir, config, true);
            }
            SpecAction::Cancel => {
                println!("Cancelled.");
                break;
            }
        }
    }
    Ok(())
}
