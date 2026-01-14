use crate::cli::TemplateAction;
use crate::templates;
use anyhow::Result;

/// Handles the `templates` subcommand for listing and using project templates.
///
/// This function routes to the appropriate template action based on the provided
/// `TemplateAction`.
///
/// # Arguments
///
/// * `action` - The template action to perform, wrapped in `TemplateAction` enum.
/// * `output_dir` - Target directory for template operations.
///
/// # Returns
///
/// Result indicating success or containing an error from template operations.
pub fn handle_templates(action: &TemplateAction, output_dir: &std::path::Path) -> Result<()> {
    match action {
        TemplateAction::List => {
            templates::list_templates();
            Ok(())
        }
        TemplateAction::Use { name } => templates::use_template(name, output_dir),
    }
}
