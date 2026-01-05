use crate::config::Config;
use crate::db::features::Feature;
use crate::utils::write_file;
use anyhow::Result;
use std::io::{self, BufRead, Write};
use std::path::Path;

use super::settings::{LoopAction, LoopSettings};

/// Generate a standard fix template
pub fn generate_fix_template(feature: &Feature, error: &str, _db_path: &Path) -> Result<()> {
    // Read template
    let template_path = Path::new("templates/commands/auto-fix.md");
    let template = if template_path.exists() {
        std::fs::read_to_string(template_path)?
    } else {
        // Fallback
        "# Regression Fix\nFix {{failing_feature}}\nError: {{error_message}}".to_string()
    };

    // Replace variables
    let content = template
        .replace("{{failing_feature}}", &feature.description)
        .replace("{{error_message}}", error)
        .replace("{{current_feature}}", "latest changes")
        .replace(
            "{{verification_command}}",
            feature.verification_command.as_deref().unwrap_or("unknown"),
        );

    // Write to active command file
    let target = Path::new(".opencode/command/auto-fix-active.md");
    write_file(target, &content)?;
    Ok(())
}

/// Generate a minimal continue template with feature context injected by supervisor.
/// This removes LLM responsibility for querying the database.
pub fn generate_continue_template(feature: &Feature) -> Result<()> {
    let content = format!(
        r#"# Implement Feature

## Your Task
Implement this feature completely:

**Feature #{}: {}**

## Acceptance Criteria
{}

## What You Do
1. Implement the feature with production-quality code
2. Write necessary tests if applicable
3. **VERIFY** that the verification command below is still correct for your implementation.
4. If the command changed (e.g. new test file path), you **MUST** update it in the database:
   `opencode-autocode db exec "UPDATE features SET verification_command = 'your-new-command' WHERE id = {}"`
5. Output `===SESSION_COMPLETE===` when implementation is done

## What Supervisor Does (NOT YOU)
The supervisor will automatically handle after your session:
- Run verification: `{}`
- Commit changes to git
- Mark feature as passing if verification succeeds

## Rules
- Do NOT run git commands (git add, git commit, git push)
- Do NOT run the verification command Yourself
- Do NOT call mark-pass
- **ALLOWED**: You may use `opencode-autocode db` commands to update your OWN feature's verification_command or steps.
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
