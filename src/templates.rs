#![allow(dead_code)]
//! Template library for project scaffolding
//!
//! Provides pre-built templates for common project types.

use anyhow::{bail, Result};
use console::style;
use dialoguer::{Input, Select};
use std::path::Path;

/// Embedded project templates
const WEB_APP_TEMPLATE: &str = include_str!("../templates/projects/web-app-fullstack.md");
const CLI_TOOL_TEMPLATE: &str = include_str!("../templates/projects/cli-tool.md");
const API_REST_TEMPLATE: &str = include_str!("../templates/projects/api-rest.md");


/// Template metadata
#[derive(Debug, Clone)]
pub struct Template {
    pub name: &'static str,
    pub display_name: &'static str,
    pub description: &'static str,
    pub content: &'static str,
}

/// Get all available templates
pub fn get_templates() -> Vec<Template> {
    vec![
        Template {
            name: "web-app-fullstack",
            display_name: "🌐 Full-Stack Web App",
            description: "React + Node.js/Express with SQLite",
            content: WEB_APP_TEMPLATE,
        },
        Template {
            name: "cli-tool",
            display_name: "🔧 CLI Tool",
            description: "Rust CLI with clap, config file support",
            content: CLI_TOOL_TEMPLATE,
        },
        Template {
            name: "api-rest",
            display_name: "🔌 REST API",
            description: "Python/FastAPI with PostgreSQL",
            content: API_REST_TEMPLATE,
        },
    ]
}

/// List all available templates
pub fn list_templates() {
    let templates = get_templates();
    
    println!("\n{}", style("📚 Available Templates").cyan().bold());
    println!("{}", style("─".repeat(50)).dim());
    
    for template in &templates {
        println!(
            "\n  {} {}",
            style(template.display_name).green().bold(),
            style(format!("({})", template.name)).dim()
        );
        println!("    {}", style(template.description).dim());
    }
    
    println!("\n{}", style("─".repeat(50)).dim());
    println!("{}", style("Use: opencode-autocode templates use <name>").dim());
    println!("{}", style("Or run with -i for interactive selection").dim());
}

/// Get a template by name
pub fn get_template_by_name(name: &str) -> Option<Template> {
    get_templates().into_iter().find(|t| t.name == name)
}

/// Use a template by name, prompting for project name and description
pub fn use_template(name: &str, output_dir: &Path) -> Result<()> {
    let template = match get_template_by_name(name) {
        Some(t) => t,
        None => {
            println!("{} Template '{}' not found.", style("Error:").red(), name);
            println!("\nAvailable templates:");
            list_templates();
            bail!("Template not found: {}", name);
        }
    };

    println!("\n{} {}", 
        style("Using template:").cyan(), 
        style(template.display_name).green().bold()
    );
    
    // Get project name
    let project_name: String = Input::new()
        .with_prompt("Project name")
        .interact_text()?;

    // Get description
    let description: String = Input::new()
        .with_prompt("Brief description")
        .interact_text()?;

    // Fill in placeholders
    let spec = template.content
        .replace("{{PROJECT_NAME}}", &project_name)
        .replace("{{DESCRIPTION}}", &description);

    // Validate the filled template
    let validation = crate::validation::validate_spec(&spec)?;
    validation.print();

    // Scaffold
    crate::scaffold::scaffold_with_spec_text(output_dir, &spec)?;
    
    println!("\n{}", style("✅ Project scaffolded from template!").green().bold());

    Ok(())
}

/// Interactive template selection
pub fn select_template_interactive(output_dir: &Path) -> Result<()> {
    let templates = get_templates();
    
    println!("\n{}", style("📚 Template Library").cyan().bold());
    println!("{}\n", style("Select a project template to get started quickly.").dim());

    let items: Vec<String> = templates
        .iter()
        .map(|t| format!("{} - {}", t.display_name, t.description))
        .collect();

    let selection = Select::new()
        .with_prompt("Choose a template")
        .items(&items)
        .default(0)
        .interact()?;

    let template = &templates[selection];
    
    // Get project name
    let project_name: String = Input::new()
        .with_prompt("Project name")
        .interact_text()?;

    // Get description
    let description: String = Input::new()
        .with_prompt("Brief description")
        .interact_text()?;

    // Fill in placeholders
    let spec = template.content
        .replace("{{PROJECT_NAME}}", &project_name)
        .replace("{{DESCRIPTION}}", &description);

    // Validate
    println!("\n{}", style("─── Validating Specification ───").cyan().bold());
    let validation = crate::validation::validate_spec(&spec)?;
    validation.print();

    // Scaffold
    crate::scaffold::scaffold_with_spec_text(output_dir, &spec)?;
    
    println!("\n{}", style("✅ Project scaffolded from template!").green().bold());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_templates_exist() {
        let templates = get_templates();
        assert_eq!(templates.len(), 3);
    }

    #[test]
    fn test_get_template_by_name() {
        let template = get_template_by_name("cli-tool");
        assert!(template.is_some());
        assert_eq!(template.unwrap().name, "cli-tool");
    }

    #[test]
    fn test_template_has_placeholders() {
        let templates = get_templates();
        for template in templates {
            assert!(template.content.contains("{{PROJECT_NAME}}"));
        }
    }
}
