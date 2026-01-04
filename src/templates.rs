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
    println!(
        "{}",
        style("Use: opencode-autocode templates use <name>").dim()
    );
    println!(
        "{}",
        style("Or run with -i for interactive selection").dim()
    );
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

    println!(
        "\n{} {}",
        style("Using template:").cyan(),
        style(template.display_name).green().bold()
    );

    // Get project name
    let project_name: String = Input::new().with_prompt("Project name").interact_text()?;

    // Get description
    let description: String = Input::new()
        .with_prompt("Brief description")
        .interact_text()?;

    // Fill in placeholders
    let spec = template
        .content
        .replace("{{PROJECT_NAME}}", &project_name)
        .replace("{{DESCRIPTION}}", &description);

    // Validate the filled template
    let validation = crate::validation::validate_spec(&spec, 0, 0)?;
    validation.print();

    // Scaffold
    crate::scaffold::scaffold_with_spec_text(output_dir, &spec)?;

    println!(
        "\n{}",
        style("✅ Project scaffolded from template!").green().bold()
    );

    Ok(())
}

/// Interactive template selection
pub fn select_template_interactive(output_dir: &Path) -> Result<()> {
    let templates = get_templates();

    println!("\n{}", style("📚 Template Library").cyan().bold());
    println!(
        "{}\n",
        style("Select a project template to get started quickly.").dim()
    );

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
    let project_name: String = Input::new().with_prompt("Project name").interact_text()?;

    // Get description
    let description: String = Input::new()
        .with_prompt("Brief description")
        .interact_text()?;

    // Fill in placeholders
    let spec = template
        .content
        .replace("{{PROJECT_NAME}}", &project_name)
        .replace("{{DESCRIPTION}}", &description);

    // Validate
    println!(
        "\n{}",
        style("─── Validating Specification ───").cyan().bold()
    );
    let validation = crate::validation::validate_spec(&spec, 0, 0)?;
    validation.print();

    // Scaffold
    crate::scaffold::scaffold_with_spec_text(output_dir, &spec)?;

    println!(
        "\n{}",
        style("✅ Project scaffolded from template!").green().bold()
    );

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

    /// We use this heuristic (text.len() / 4) to estimate token usage during tests.
    /// While not perfect, it helps us catch massive regressions in prompt size before they hit production.
    fn estimate_tokens(text: &str) -> usize {
        (text.len() + 3) / 4
    }

    /// We print these counts to stdout so developers can see the "context tax" of each template.
    #[test]
    fn test_project_template_token_counts() {
        println!("\n=== PROJECT TEMPLATE TOKEN COUNTS ===\n");

        let templates = get_templates();
        let mut total_tokens = 0;

        for template in &templates {
            let chars = template.content.len();
            let tokens = estimate_tokens(template.content);
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
        const AUTO_INIT: &str = include_str!("../templates/commands/auto-init.md");
        const AUTO_CONTINUE: &str = include_str!("../templates/commands/auto-continue.md");
        const AUTO_ENHANCE: &str = include_str!("../templates/commands/auto-enhance.md");

        // Core modules that get included
        const CORE_IDENTITY: &str = include_str!("../templates/core/identity.md");
        const CORE_SECURITY: &str = include_str!("../templates/core/security.md");
        const CORE_DATABASE: &str = include_str!("../templates/core/database.md");
        const CORE_MCP_GUIDE: &str = include_str!("../templates/core/mcp_guide.md");
        const CORE_SIGNALING: &str = include_str!("../templates/core/signaling.md");
        const CORE_COMMUNICATION: &str = include_str!("../templates/core/communication.md");

        println!("\n=== COMMAND TEMPLATE TOKEN COUNTS ===\n");
        println!("--- Core Modules (included via {{{{INCLUDE}}}}) ---\n");

        let core_modules = [
            ("core/identity.md", CORE_IDENTITY),
            ("core/security.md", CORE_SECURITY),
            ("core/database.md", CORE_DATABASE),
            ("core/mcp_guide.md", CORE_MCP_GUIDE),
            ("core/signaling.md", CORE_SIGNALING),
            ("core/communication.md", CORE_COMMUNICATION),
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
            ("auto-init.md", AUTO_INIT),
            ("auto-continue.md", AUTO_CONTINUE),
            ("auto-enhance.md", AUTO_ENHANCE),
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

        // Simple include resolver for testing
        fn resolve_includes_test(template: &str) -> String {
            let mut result = template.to_string();
            let includes: &[(&str, &str)] = &[
                (
                    "core/identity.md",
                    include_str!("../templates/core/identity.md"),
                ),
                (
                    "core/security.md",
                    include_str!("../templates/core/security.md"),
                ),
                (
                    "core/database.md",
                    include_str!("../templates/core/database.md"),
                ),
                (
                    "core/mcp_guide.md",
                    include_str!("../templates/core/mcp_guide.md"),
                ),
                (
                    "core/signaling.md",
                    include_str!("../templates/core/signaling.md"),
                ),
                (
                    "core/communication.md",
                    include_str!("../templates/core/communication.md"),
                ),
            ];
            for (path, content) in includes {
                let directive = format!("{{{{INCLUDE {}}}}}", path);
                result = result.replace(&directive, content);
            }
            result
        }

        let commands_resolved = [
            ("auto-init.md (resolved)", resolve_includes_test(AUTO_INIT)),
            (
                "auto-continue.md (resolved)",
                resolve_includes_test(AUTO_CONTINUE),
            ),
            (
                "auto-enhance.md (resolved)",
                resolve_includes_test(AUTO_ENHANCE),
            ),
        ];

        for (name, content) in &commands_resolved {
            let tokens = estimate_tokens(&content);
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
        const GENERATOR_PROMPT: &str = include_str!("../templates/generator_prompt.md");

        println!("\n=== GENERATOR PROMPT TOKEN COUNT ===\n");

        let tokens = estimate_tokens(GENERATOR_PROMPT);
        println!(
            "{:<30} {:>6} chars  ~{:>5} tokens",
            "generator_prompt.md",
            GENERATOR_PROMPT.len(),
            tokens
        );
        println!();
    }
}
