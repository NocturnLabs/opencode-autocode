use anyhow::Result;
use console::style;
use dialoguer::{Confirm, Editor, FuzzySelect, Input, MultiSelect, Select};
use std::fs;
use std::io::Write;
use std::path::Path;

use crate::config_tui::{fetch_available_models, run_config_tui};
use crate::generator::{generate_spec_from_idea, refine_spec_from_idea};
use crate::scaffold::{
    scaffold_custom, scaffold_default, scaffold_from_spec, scaffold_with_spec_text,
};
use crate::spec::{AppSpec, Feature, Priority, TechStack};
use crate::validation::{print_diff, validate_spec};

/// Interactive mode options
enum InteractiveMode {
    Generated,
    Manual,
    FromSpecFile,
    Default,
}

/// Actions available after spec generation
enum SpecAction {
    Accept,
    Edit,
    SaveToFile,
    Refine,
    Regenerate,
    Cancel,
}

// ─────────────────────────────────────────────────────────────────────────────
// Entry Point
// ─────────────────────────────────────────────────────────────────────────────

/// Run the interactive TUI to build an app spec
pub fn run_interactive(output_dir: &Path) -> Result<()> {
    print_header();

    let mode = select_mode()?;

    match mode {
        InteractiveMode::Generated => run_generated_mode(output_dir, None),
        InteractiveMode::Manual => run_manual_mode(output_dir),
        InteractiveMode::FromSpecFile => run_from_spec_file_mode(output_dir),
        InteractiveMode::Default => run_default_mode(output_dir),
    }
}

fn print_header() {
    println!(
        "\n{}",
        style("═══════════════════════════════════════════════════").cyan()
    );
    println!(
        "{}",
        style("  OpenCode Autonomous Plugin - Interactive Setup")
            .cyan()
            .bold()
    );
    println!(
        "{}\n",
        style("═══════════════════════════════════════════════════").cyan()
    );
}

fn select_mode() -> Result<InteractiveMode> {
    let mode_idx = Select::new()
        .with_prompt("How would you like to create your project spec?")
        .items([
            "🤖 Generated - AI researches and creates full spec",
            "📝 Manual - Fill out project details step by step",
            "📁 From file - Use an existing app_spec.md",
            "⚡ Default - Use built-in specification",
        ])
        .default(0)
        .interact()?;

    Ok(match mode_idx {
        0 => InteractiveMode::Generated,
        1 => InteractiveMode::Manual,
        2 => InteractiveMode::FromSpecFile,
        _ => InteractiveMode::Default,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Spec File Mode
// ─────────────────────────────────────────────────────────────────────────────

fn run_from_spec_file_mode(output_dir: &Path) -> Result<()> {
    println!("\n{}", style("─── Spec File Mode ───").yellow().bold());

    let spec_path: String = Input::new()
        .with_prompt("Path to spec file")
        .default("app_spec.md".to_string())
        .interact_text()?;

    let spec_path = std::path::PathBuf::from(&spec_path);
    if !spec_path.exists() {
        println!("{} Spec file not found.", style("Error:").red().bold());
        return Ok(());
    }

    scaffold_custom(output_dir, &spec_path)?;
    println!(
        "\n{}",
        style("✅ Project scaffolded from spec file!")
            .green()
            .bold()
    );
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Default Mode
// ─────────────────────────────────────────────────────────────────────────────

fn run_default_mode(output_dir: &Path) -> Result<()> {
    println!("\n{}", style("─── Default Mode ───").yellow().bold());
    println!(
        "{}",
        style("Using the built-in default specification.").dim()
    );

    if Confirm::new()
        .with_prompt("Scaffold project with default spec?")
        .default(true)
        .interact()?
    {
        scaffold_default(output_dir)?;
        println!(
            "\n{}",
            style("✅ Project scaffolded with default spec!")
                .green()
                .bold()
        );
    } else {
        println!("{}", style("Cancelled.").red());
    }
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Generated Mode (Main Loop)
// ─────────────────────────────────────────────────────────────────────────────

fn run_generated_mode(output_dir: &Path, initial_model: Option<&str>) -> Result<()> {
    println!("\n{}", style("─── AI Spec Generation ───").yellow().bold());
    println!(
        "{}",
        style("Describe your project and AI will create a comprehensive spec.").dim()
    );

    let model_owned = prompt_for_model(initial_model)?;
    let model = model_owned.as_deref();

    let idea = prompt_for_idea()?;
    if idea.is_empty() {
        return Ok(());
    }

    let mut spec_text = match generate_initial_spec(&idea, model) {
        Ok(spec) => spec,
        Err(e) => return handle_generation_error(e, output_dir),
    };

    run_validation_loop(output_dir, &mut spec_text, model_owned)
}

fn prompt_for_model(initial: Option<&str>) -> Result<Option<String>> {
    // Fetch available models
    let models = match fetch_available_models() {
        Ok(m) => m,
        Err(_) => {
            // Fall back to manual input if fetch fails
            println!(
                "{}",
                style("Could not fetch models, enter manually").yellow()
            );
            let input: String = Input::new()
                .with_prompt("Model (leave empty for default)")
                .default(initial.unwrap_or("").to_string())
                .allow_empty(true)
                .interact_text()?;
            return Ok(if input.trim().is_empty() {
                None
            } else {
                Some(input.trim().to_string())
            });
        }
    };

    println!(
        "{}",
        style(format!("Found {} available models", models.len())).dim()
    );

    let mut options = vec!["(Use default model)".to_string()];
    options.extend(models);
    options.push("(Enter custom model)".to_string());

    let default_idx = if let Some(init) = initial {
        options.iter().position(|m| m == init).unwrap_or(0)
    } else {
        0
    };

    let idx = FuzzySelect::new()
        .with_prompt("Select model (type to filter)")
        .items(&options)
        .default(default_idx)
        .max_length(15)
        .interact()?;

    if idx == 0 {
        Ok(None) // Use default
    } else if idx == options.len() - 1 {
        let custom: String = Input::new()
            .with_prompt("Enter model (provider/model)")
            .default(initial.unwrap_or("").to_string())
            .interact_text()?;
        Ok(Some(custom))
    } else {
        Ok(Some(options[idx].clone()))
    }
}

fn prompt_for_idea() -> Result<String> {
    let idea: String = Input::new()
        .with_prompt("Describe your project idea")
        .interact_text()?;

    if idea.trim().is_empty() {
        println!("{}", style("No idea provided.").red());
        return Ok(String::new());
    }
    Ok(idea)
}

fn generate_initial_spec(idea: &str, model: Option<&str>) -> Result<String> {
    // Clear any lingering line content from input echo
    print!("\x1B[2K\r");
    let _ = std::io::stdout().flush();

    println!(
        "\n{}",
        style("─────────────────────────────────────────────").dim()
    );

    generate_spec_from_idea(idea, model, |msg| {
        print!("{}", msg);
        let _ = std::io::stdout().flush();
    })
}

fn handle_generation_error(e: anyhow::Error, output_dir: &Path) -> Result<()> {
    println!("\n{} {}", style("Error:").red().bold(), e);

    if Confirm::new()
        .with_prompt("Switch to manual mode?")
        .default(true)
        .interact()?
    {
        run_manual_mode(output_dir)
    } else {
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Validation Loop
// ─────────────────────────────────────────────────────────────────────────────

fn run_validation_loop(
    output_dir: &Path,
    spec_text: &mut String,
    model_owned: Option<String>,
) -> Result<()> {
    loop {
        let validation = validate_and_preview(spec_text)?;
        let action = prompt_for_action(validation.is_valid)?;

        match action {
            SpecAction::Accept => {
                if handle_accept(output_dir, spec_text, validation.is_valid)? {
                    break;
                }
            }
            SpecAction::Edit => {
                handle_edit(spec_text)?;
            }
            SpecAction::SaveToFile => {
                save_spec_to_file(output_dir, spec_text)?;
                break;
            }
            SpecAction::Refine => {
                handle_refine(spec_text, model_owned.as_deref())?;
            }
            SpecAction::Regenerate => {
                return run_generated_mode(output_dir, model_owned.as_deref());
            }
            SpecAction::Cancel => {
                println!("{}", style("Cancelled.").red());
                break;
            }
        }
    }
    Ok(())
}

fn validate_and_preview(spec_text: &str) -> Result<crate::validation::ValidationResult> {
    println!(
        "\n{}",
        style("─── Validating Specification ───").cyan().bold()
    );

    let validation = validate_spec(spec_text)?;
    validation.print();

    if !validation.is_valid {
        println!("\n{}", style("The spec has validation errors.").red());
    }

    // Show preview
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

    for line in spec_text.lines().take(25) {
        println!("  {}", style(line).dim());
    }
    let total_lines = spec_text.lines().count();
    if total_lines > 25 {
        println!(
            "  {}",
            style(format!("... ({} more lines)", total_lines - 25)).dim()
        );
    }

    Ok(validation)
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

fn handle_accept(output_dir: &Path, spec_text: &str, is_valid: bool) -> Result<bool> {
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

    // Prompt to configure settings
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

fn handle_edit(spec_text: &mut String) -> Result<()> {
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

fn save_spec_to_file(output_dir: &Path, spec_text: &str) -> Result<()> {
    let spec_path = output_dir.join("app_spec.md");
    fs::write(&spec_path, spec_text)?;
    println!(
        "\n{} {}",
        style("📄 Saved to:").cyan(),
        style(spec_path.display()).green()
    );

    // Print next steps hint
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

fn handle_refine(spec_text: &mut String, model: Option<&str>) -> Result<()> {
    println!(
        "\n{}",
        style("─── Refine Specification ───").yellow().bold()
    );

    // Show current spec with line numbers for targeted instructions
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
            // Show diff of changes
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

// ─────────────────────────────────────────────────────────────────────────────
// Manual Mode
// ─────────────────────────────────────────────────────────────────────────────

fn run_manual_mode(output_dir: &Path) -> Result<()> {
    println!(
        "\n{}",
        style("─── Manual Spec Creation ───").yellow().bold()
    );

    let spec = collect_manual_spec()?;
    print_manual_summary(&spec);

    if Confirm::new()
        .with_prompt("Generate plugin files?")
        .default(true)
        .interact()?
    {
        scaffold_from_spec(output_dir, &spec)?;
    } else {
        println!("{}", style("Cancelled.").red());
    }

    Ok(())
}

fn collect_manual_spec() -> Result<AppSpec> {
    let project_name: String = Input::new().with_prompt("Project name").interact_text()?;
    let overview: String = Input::new()
        .with_prompt("Brief description")
        .interact_text()?;

    let technology = if Confirm::new()
        .with_prompt("Define technology stack?")
        .default(true)
        .interact()?
    {
        Some(collect_tech_stack()?)
    } else {
        None
    };

    let features = collect_features()?;
    let success_criteria = collect_success_criteria()?;

    Ok(AppSpec {
        project_name,
        overview,
        features,
        success_criteria,
        technology,
        database: None,
        api_endpoints: None,
    })
}

fn collect_features() -> Result<Vec<Feature>> {
    println!("\n{}", style("Add Features").yellow().bold());
    println!(
        "{}",
        style("(Enter features one at a time, empty to finish)").dim()
    );

    let mut features = Vec::new();
    loop {
        let name: String = Input::new()
            .with_prompt("Feature name (empty to finish)")
            .allow_empty(true)
            .interact_text()?;

        if name.is_empty() {
            break;
        }

        let description: String = Input::new().with_prompt("Description").interact_text()?;

        let priority = select_priority()?;

        features.push(Feature {
            name,
            description,
            priority,
            sub_features: Vec::new(),
        });

        println!("{} Feature added!", style("✓").green());
    }
    Ok(features)
}

fn select_priority() -> Result<Priority> {
    let idx = Select::new()
        .with_prompt("Priority")
        .items(["Critical", "High", "Medium", "Low"])
        .default(2)
        .interact()?;

    Ok(match idx {
        0 => Priority::Critical,
        1 => Priority::High,
        2 => Priority::Medium,
        _ => Priority::Low,
    })
}

fn collect_success_criteria() -> Result<Vec<String>> {
    println!("\n{}", style("Success Criteria").yellow().bold());
    println!(
        "{}",
        style("(Enter criteria one at a time, empty to finish)").dim()
    );

    let mut criteria = Vec::new();
    loop {
        let criterion: String = Input::new()
            .with_prompt("Criterion (empty to finish)")
            .allow_empty(true)
            .interact_text()?;

        if criterion.is_empty() {
            break;
        }
        criteria.push(criterion);
    }
    Ok(criteria)
}

fn print_manual_summary(spec: &AppSpec) {
    println!(
        "\n{}",
        style("═══════════════════════════════════════════════════").cyan()
    );
    println!("{}", style("  Summary").cyan().bold());
    println!(
        "{}",
        style("═══════════════════════════════════════════════════").cyan()
    );
    println!("  Project: {}", style(&spec.project_name).green());
    println!("  Features: {}", style(spec.features.len()).yellow());
    println!(
        "  Criteria: {}",
        style(spec.success_criteria.len()).yellow()
    );
    println!();
}

// ─────────────────────────────────────────────────────────────────────────────
// Tech Stack Collection
// ─────────────────────────────────────────────────────────────────────────────

fn collect_tech_stack() -> Result<TechStack> {
    let languages = collect_with_other(
        "Languages (space to select)",
        &["Rust", "TypeScript", "JavaScript", "Python", "Go", "Other"],
        "Other languages (comma-separated)",
    )?;

    let frameworks = collect_with_other(
        "Frameworks (space to select)",
        &[
            "React", "Next.js", "Vue", "Svelte", "Express", "Actix", "Axum", "FastAPI", "Django",
            "Gin", "Other",
        ],
        "Other frameworks (comma-separated)",
    )?;

    Ok(TechStack {
        languages,
        frameworks,
        tools: Vec::new(),
    })
}

fn collect_with_other(prompt: &str, options: &[&str], other_prompt: &str) -> Result<Vec<String>> {
    let indices = MultiSelect::new()
        .with_prompt(prompt)
        .items(options)
        .interact()?;

    let mut items: Vec<String> = indices.iter().map(|&i| options[i].to_string()).collect();

    if items.contains(&"Other".to_string()) {
        items.retain(|s| s != "Other");
        let custom: String = Input::new()
            .with_prompt(other_prompt)
            .allow_empty(true)
            .interact_text()?;
        for item in custom.split(',') {
            let item = item.trim();
            if !item.is_empty() {
                items.push(item.to_string());
            }
        }
    }

    Ok(items)
}
