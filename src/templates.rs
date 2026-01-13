//! Template library for project scaffolding
//!
//! Provides pre-built templates for common project types.

use anyhow::{bail, Result};
use std::path::Path;

use crate::template_xml;
use crate::tui::prompts::input;

/// Embedded project templates
const WEB_APP_TEMPLATE: &str = include_str!("../templates/projects/web-app-fullstack.xml");
const CLI_TOOL_TEMPLATE: &str = include_str!("../templates/projects/cli-tool.xml");
const API_REST_TEMPLATE: &str = include_str!("../templates/projects/api-rest.xml");

/// Template metadata
#[derive(Debug, Clone)]
pub struct Template {
    pub name: &'static str,
    pub display_name: &'static str,
    pub description: &'static str,
    pub content: String,
}

/// Get all available templates
pub fn get_templates() -> Result<Vec<Template>> {
    Ok(vec![
        Template {
            name: "web-app-fullstack",
            display_name: "🌐 Full-Stack Web App",
            description: "React + Node.js/Express with SQLite",
            content: template_xml::render_template(WEB_APP_TEMPLATE)?,
        },
        Template {
            name: "cli-tool",
            display_name: "🔧 CLI Tool",
            description: "Rust CLI with clap, config file support",
            content: template_xml::render_template(CLI_TOOL_TEMPLATE)?,
        },
        Template {
            name: "api-rest",
            display_name: "🔌 REST API",
            description: "Python/FastAPI with PostgreSQL",
            content: template_xml::render_template(API_REST_TEMPLATE)?,
        },
    ])
}

/// List all available templates
pub fn list_templates() {
    let templates = match get_templates() {
        Ok(templates) => templates,
        Err(err) => {
            println!("Error loading templates: {}", err);
            return;
        }
    };

    println!("\n📚 Available Templates");
    println!("{}", "─".repeat(50));

    for template in &templates {
        println!("\n  {} ({})", template.display_name, template.name);
        println!("    {}", template.description);
    }

    println!("\n{}", "─".repeat(50));
    println!("Use: opencode-forger templates use <name>");
    println!("Or run with -i for interactive selection");
}

/// Get a template by name
pub fn get_template_by_name(name: &str) -> Result<Option<Template>> {
    Ok(get_templates()?.into_iter().find(|t| t.name == name))
}

/// Use a template by name, prompting for project name and description
pub fn use_template(name: &str, output_dir: &Path) -> Result<()> {
    let template = match get_template_by_name(name)? {
        Some(t) => t,
        None => {
            println!("Error: Template '{}' not found.", name);
            println!("\nAvailable templates:");
            list_templates();
            bail!("Template not found: {}", name);
        }
    };

    println!("\nUsing template: {}", template.display_name);

    // Get project name
    let project_name = input("Project name", None)?;

    // Get description
    let description = input("Brief description", None)?;

    // Fill in placeholders
    let spec = template
        .content
        .replace("{{PROJECT_NAME}}", &project_name)
        .replace("{{DESCRIPTION}}", &description);

    // Validate the filled template
    let validation = crate::validation::validate_spec(&spec)?;
    validation.print();

    // Scaffold
    crate::services::scaffold::scaffold_with_spec_text(output_dir, &spec)?;

    println!("\n✅ Project scaffolded from template!");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_templates_exist() {
        let templates = get_templates().unwrap();
        assert_eq!(templates.len(), 3);
    }

    #[test]
    fn test_get_template_by_name() {
        let template = get_template_by_name("cli-tool").unwrap();
        assert!(template.is_some());
        assert_eq!(template.unwrap().name, "cli-tool");
    }

    #[test]
    fn test_template_has_placeholders() {
        let templates = get_templates().unwrap();
        for template in templates {
            assert!(template.content.contains("{{PROJECT_NAME}}"));
        }
    }

    /// We use this heuristic (text.len() / 4) to estimate token usage during tests.
    /// While not perfect, it helps us catch massive regressions in prompt size before they hit production.
    fn estimate_tokens(text: &str) -> usize {
        text.len().div_ceil(4)
    }

    /// We print these counts to stdout so developers can see the "context tax" of each template.
    #[test]
    fn test_project_template_token_counts() {
        println!("\n=== PROJECT TEMPLATE TOKEN COUNTS ===\n");

        let templates = get_templates().unwrap();
        let mut total_tokens = 0;

        for template in &templates {
            let chars = template.content.len();
            let tokens = estimate_tokens(&template.content);
            total_tokens += tokens;

            println!(
                "{:<25} {:>6} chars  ~{:>5} tokens",
                template.name, chars, tokens
            );
        }

        println!("\n{:<25} {:>14} ~{:>5} tokens", "TOTAL", "", total_tokens);
        println!();
    }

    /// Reports token counts for command templates (auto-init, auto-continue, etc.)
    /// These are the templates that get injected into the agent context
    #[test]
    fn test_command_template_token_counts() {
        // Import the command templates from scaffold module
        const AUTO_INIT: &str = include_str!("../templates/commands/auto-init.xml");
        const AUTO_CONTINUE: &str = include_str!("../templates/commands/auto-continue.xml");
        const AUTO_ENHANCE: &str = include_str!("../templates/commands/auto-enhance.xml");

        // Core modules that get included
        const CORE_IDENTITY: &str = include_str!("../templates/core/identity.xml");
        const CORE_SECURITY: &str = include_str!("../templates/core/security.xml");
        const CORE_DATABASE: &str = include_str!("../templates/core/database.xml");
        const CORE_MCP_GUIDE: &str = include_str!("../templates/core/mcp_guide.xml");
        const CORE_SIGNALING: &str = include_str!("../templates/core/signaling.xml");

        let auto_init = template_xml::render_template(AUTO_INIT).unwrap();
        let auto_continue = template_xml::render_template(AUTO_CONTINUE).unwrap();
        let auto_enhance = template_xml::render_template(AUTO_ENHANCE).unwrap();

        let core_identity = template_xml::render_template(CORE_IDENTITY).unwrap();
        let core_security = template_xml::render_template(CORE_SECURITY).unwrap();
        let core_database = template_xml::render_template(CORE_DATABASE).unwrap();
        let core_mcp_guide = template_xml::render_template(CORE_MCP_GUIDE).unwrap();
        let core_signaling = template_xml::render_template(CORE_SIGNALING).unwrap();

        println!("\n=== COMMAND TEMPLATE TOKEN COUNTS ===\n");
        println!("--- Core Modules (included via {{{{INCLUDE}}}}) ---\n");

        let core_modules = [
            ("core/identity.xml", &core_identity),
            ("core/security.xml", &core_security),
            ("core/database.xml", &core_database),
            ("core/mcp_guide.xml", &core_mcp_guide),
            ("core/signaling.xml", &core_signaling),
        ];

        for (name, content) in &core_modules {
            let tokens = estimate_tokens(content);
            println!(
                "{:<30} {:>6} chars  ~{:>5} tokens",
                name,
                content.len(),
                tokens
            );
        }

        println!("\n--- Command Templates (Raw, before include resolution) ---\n");

        let commands_raw = [
            ("auto-init.xml", &auto_init),
            ("auto-continue.xml", &auto_continue),
            ("auto-enhance.xml", &auto_enhance),
        ];

        for (name, content) in &commands_raw {
            let tokens = estimate_tokens(content);
            println!(
                "{:<30} {:>6} chars  ~{:>5} tokens",
                name,
                content.len(),
                tokens
            );
        }

        println!("\n--- Command Templates (Resolved, after include resolution) ---\n");

        // Use the public resolve_includes from scaffold module
        let commands_resolved = [
            (
                "auto-init.xml (resolved)",
                crate::services::scaffold::resolve_includes(&auto_init).unwrap(),
            ),
            (
                "auto-continue.xml (resolved)",
                crate::services::scaffold::resolve_includes(&auto_continue).unwrap(),
            ),
            (
                "auto-enhance.xml (resolved)",
                crate::services::scaffold::resolve_includes(&auto_enhance).unwrap(),
            ),
        ];

        for (name, content) in &commands_resolved {
            let tokens = estimate_tokens(content);
            println!(
                "{:<30} {:>6} chars  ~{:>5} tokens",
                name,
                content.len(),
                tokens
            );
        }

        println!();
    }

    /// Reports token count for the generator prompt
    #[test]
    fn test_generator_prompt_token_count() {
        const GENERATOR_PROMPT: &str = include_str!("../templates/generator_prompt.xml");

        println!("\n=== GENERATOR PROMPT TOKEN COUNT ===\n");

        let generator_prompt = template_xml::render_template(GENERATOR_PROMPT).unwrap();
        let tokens = estimate_tokens(&generator_prompt);
        println!(
            "{:<30} {:>6} chars  ~{:>5} tokens",
            "generator_prompt.xml",
            generator_prompt.len(),
            tokens
        );
        println!();
    }
}
