use crate::config::Config;
use crate::db::features::Feature;
use crate::template_xml;
use crate::utils::write_file;
use anyhow::Result;
use std::io::{self, BufRead, Write};
use std::path::Path;

use super::settings::{LoopAction, LoopSettings};

/// Generate a standard fix template
pub fn generate_fix_template(
    feature: &Feature,
    error: &str,
    _db_path: &Path,
    dual_model: bool,
) -> Result<()> {
    // Read template
    let template_path = Path::new("templates/commands/auto-fix.xml");
    let template = if template_path.exists() {
        let raw_template = std::fs::read_to_string(template_path)?;
        template_xml::render_template(&raw_template).unwrap_or(raw_template)
    } else {
        // Fallback
        "# Regression Fix\nFix {{failing_feature}}\nError: {{error_message}}\n\n{{dual_model_instructions}}\n\n{{explore_instructions}}".to_string()
    };

    let dual_model_instructions = if dual_model {
        "\n> **Dual Model**: Delegate code changes to `@coder`."
    } else {
        ""
    };

    let explore_msg = "Use `@explore` to understand the failure context.";

    // Replace variables
    let content = template
        .replace("{{failing_feature}}", &feature.description)
        .replace("{{error_message}}", error)
        .replace("{{current_feature}}", "latest changes")
        .replace("{{dual_model_instructions}}", dual_model_instructions)
        .replace("{{explore_instructions}}", explore_msg)
        .replace(
            "{{verification_command}}",
            feature.verification_command.as_deref().unwrap_or("unknown"),
        );

    // Resolve includes (e.g. core/database.md)
    let content = crate::services::scaffold::resolve_includes(&content)?;

    // Write to active command file
    let target = Path::new(".opencode/command/auto-fix-active.md");
    write_file(target, &content)?;
    Ok(())
}

/// Generate a minimal continue template with feature context injected by supervisor.
/// This removes LLM responsibility for querying the database.
/// Generate a minimal continue template with feature context injected by supervisor.
/// This removes LLM responsibility for querying the database.
pub fn generate_continue_template(feature: &Feature, dual_model: bool) -> Result<()> {
    let dual_model_section = if dual_model {
        "\n## Dual Model Architecture\nYou are the **Reasoning Agent**. Plan the solution and delegate implementation to `@coder`.\n"
    } else {
        ""
    };

    let content = format!(
        r#"# Implement Feature

## Your Task
Implement this feature completely:

**Feature #{}: {}**

## Acceptance Criteria
{}
{}

## 🛑 MANDATORY: GET YOUR BEARINGS
1. Read `app_spec.md` to refresh context.
2. Run `opencode-forger db stats` to see overall progress.
3. **REGRESSION CHECK**: Run 1-2 of the features marked as passing to verify they still work.
4. If you find ANY regressions, fix them BEFORE starting the new feature.

## 🚀 MINIMAL WORK SESSION
Your goal is to be **FAST**. 
1. Implement **ONLY** Feature #{}.
2. Verify it end-to-end.
3. Output `===SESSION_COMPLETE===` immediately.
Do not over-engineer or explore unrelated files.

## What You Do
1. **Use `@explore` to understand the codebase context.** 
2. Implement the feature with production-quality code.
3. Write necessary tests if applicable.
4. **VERIFY** that the verification command below is still correct for your implementation.
5. If the command changed (e.g. new test file path), you **MUST** update it in the database:
   `opencode-forger db exec "UPDATE features SET verification_command = 'your-new-command' WHERE id = {}"`
6. Output `===SESSION_COMPLETE===` when implementation is done

## What Supervisor Does (NOT YOU)
The supervisor will automatically handle after your session:
- Run verification: `{}`
- Commit changes to git
- Mark feature as passing if verification succeeds

## Rules
- Do NOT run git commands (git add, git commit, git push)
- Do NOT run the verification command Yourself
- Do NOT call mark-pass
- **ALLOWED**: You may use `opencode-forger db` commands to update your OWN feature's verification_command or steps.
- ONLY implement this one feature and output ===SESSION_COMPLETE===
"#,
        feature.id.unwrap_or(0),
        feature.description,
        if feature.steps.is_empty() {
            "Not specified - implement as described".to_string()
        } else {
            feature
                .steps
                .iter()
                .enumerate()
                .map(|(i, s)| format!("{}. {}", i + 1, s))
                .collect::<Vec<_>>()
                .join("\n")
        },
        dual_model_section,
        feature.id.unwrap_or(0),
        feature.id.unwrap_or(0),
        feature
            .verification_command
            .as_deref()
            .unwrap_or("# No verification command specified")
    );

    let target = Path::new(".opencode/command/auto-continue-active.md");
    write_file(target, &content)?;
    Ok(())
}

/// Handle enhancement phase input and generation
pub fn handle_enhancement_phase(
    _db_path: &Path,
    _config: &Config,
    _settings: &LoopSettings,
    _iteration: usize,
) -> Result<LoopAction> {
    println!("\n✨ All features complete! The autonomous loop is now in enhancement mode.");
    println!("What would you like to enhance? (or type 'exit' to finish)");
    print!("> ");
    io::stdout().flush()?;

    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut enhancement_request = String::new();
    reader.read_line(&mut enhancement_request)?;

    let enhancement_request = enhancement_request.trim();

    if enhancement_request.is_empty() || enhancement_request.to_lowercase() == "exit" {
        return Ok(LoopAction::Break);
    }

    // Generate dynamic enhancement template
    let template = r#"# Enhancement Request
{{enhancement_request}}

Please implement this enhancement for the current project.
"#;
    let content = template.replace("{{enhancement_request}}", enhancement_request);

    // Write to active command file
    let target = Path::new(".opencode/command/auto-enhance-active.md");
    write_file(target, &content)?;

    Ok(LoopAction::Continue)
}
