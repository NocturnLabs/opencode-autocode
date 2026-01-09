use crate::cli::TemplateAction;
use crate::templates;
use anyhow::Result;

pub fn handle_templates(action: &TemplateAction, output_dir: &std::path::Path) -> Result<()> {
    match action {
        TemplateAction::List => {
            templates::list_templates();
            Ok(())
        }
        TemplateAction::Use { name } => templates::use_template(name, output_dir),
    }
}
