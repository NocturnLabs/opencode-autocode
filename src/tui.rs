//! Interactive TUI for building app spec

use anyhow::Result;
use console::style;
use dialoguer::{Confirm, Editor, Input, MultiSelect, Select};
use std::fs;
use std::path::Path;

use crate::generator::{generate_spec_from_idea, refine_spec_from_idea};
use crate::scaffold::{scaffold_default, scaffold_from_spec, scaffold_with_spec_text, scaffold_custom};
use crate::spec::{AppSpec, Feature, Priority, TechStack};
use crate::validation::{print_diff, validate_spec};

/// Interactive mode options
enum InteractiveMode {
    /// Generate spec from an idea using AI
    Generated,
    /// Manual form-based spec creation
    Manual,
    /// Use an existing spec file
    FromSpecFile,
    /// Use the built-in default spec
    Default,
}

/// Run the interactive TUI to build an app spec
pub fn run_interactive(output_dir: &Path) -> Result<()> {
    println!("\n{}", style("═══════════════════════════════════════════════════").cyan());
    println!("{}", style("  OpenCode Autonomous Plugin - Interactive Setup").cyan().bold());
    println!("{}\n", style("═══════════════════════════════════════════════════").cyan());

    // Mode selection
    let mode_idx = Select::new()
        .with_prompt("How would you like to create your project spec?")
        .items([
            "🤖 Generated (idea-based) - AI researches and creates full spec",
            "📝 Manual (form-based) - Fill out project details step by step",
            "📁 From spec file - Use an existing app_spec.md file",
            "⚡ Default - Use the built-in default specification",
        ])
        .default(0)
        .interact()?;

    let mode = match mode_idx {
        0 => InteractiveMode::Generated,
        1 => InteractiveMode::Manual,
        2 => InteractiveMode::FromSpecFile,
        _ => InteractiveMode::Default,
    };

    match mode {
        InteractiveMode::Generated => run_generated_mode(output_dir, None),
        InteractiveMode::Manual => run_manual_mode(output_dir),
        InteractiveMode::FromSpecFile => run_from_spec_file_mode(output_dir),
        InteractiveMode::Default => run_default_mode(output_dir),
    }
}

/// Run from an existing spec file
fn run_from_spec_file_mode(output_dir: &Path) -> Result<()> {
    println!("\n{}", style("─── Spec File Mode ───").yellow().bold());
    
    let spec_path: String = Input::new()
        .with_prompt("Path to spec file")
        .default("app_spec.md".to_string())
        .interact_text()?;
    
    let spec_path = std::path::PathBuf::from(&spec_path);
    
    if !spec_path.exists() {
        println!("{} {}", style("Error:").red().bold(), style("Spec file not found.").red());
        return Ok(());
    }
    
    scaffold_custom(output_dir, &spec_path)?;
    println!("\n{}", style("✅ Project scaffolded from spec file!").green().bold());
    
    Ok(())
}

/// Run with the default built-in spec
fn run_default_mode(output_dir: &Path) -> Result<()> {
    println!("\n{}", style("─── Default Mode ───").yellow().bold());
    println!("{}", style("Using the built-in default specification.").dim());
    
    let confirm = Confirm::new()
        .with_prompt("Scaffold project with default spec?")
        .default(true)
        .interact()?;
    
    if confirm {
        scaffold_default(output_dir)?;
        println!("\n{}", style("✅ Project scaffolded with default spec!").green().bold());
    } else {
        println!("{}", style("Cancelled.").red());
    }
    
    Ok(())
}

/// Run the AI-generated spec mode with validation and refinement
fn run_generated_mode(output_dir: &Path, initial_model: Option<&str>) -> Result<()> {
    println!("\n{}", style("─── AI Spec Generation ───").yellow().bold());
    println!("{}", style("Describe your project idea and the AI will research and create a comprehensive spec.").dim());
    println!("{}\n", style("The AI will use web search and documentation tools to find best practices.").dim());

    // Ask for optional custom model
    let model_input: String = Input::new()
        .with_prompt("Model to use (leave empty for default)")
        .default(initial_model.unwrap_or("").to_string())
        .allow_empty(true)
        .interact_text()?;
    
    let model: Option<&str> = if model_input.trim().is_empty() {
        None
    } else {
        Some(model_input.trim())
    };

    // Get the user's idea 
    let idea: String = Input::new()
        .with_prompt("Describe your project idea")
        .interact_text()?;

    if idea.trim().is_empty() {
        println!("{}", style("No idea provided. Exiting.").red());
        return Ok(());
    }

    // Add clear separator before generation output
    println!("\n{}", style("─────────────────────────────────────────────").dim());

    // Generate the spec using OpenCode with model
    use std::io::Write;
    let mut spec_text = match generate_spec_from_idea(&idea, model, |msg| {
        print!("{}", msg);
        // Flush to ensure output appears immediately
        let _ = std::io::stdout().flush();
    }) {
        Ok(spec) => spec,
        Err(e) => {
            println!("\n{} {}", style("Error:").red().bold(), e);
            println!("{}", style("Would you like to try manual mode instead?").dim());
            
            let try_manual = Confirm::new()
                .with_prompt("Switch to manual mode?")
                .default(true)
                .interact()?;

            if try_manual {
                return run_manual_mode(output_dir);
            } else {
                return Ok(());
            }
        }
    };

    // Store model for potential regeneration/refinement
    let model_owned = model.map(|s| s.to_string());

    // Validation and refinement loop
    loop {
        // Validate the spec
        println!("\n{}", style("─── Validating Specification ───").cyan().bold());
        
        let validation = validate_spec(&spec_text)?;
        validation.print();

        if !validation.is_valid {
            println!("\n{}", style("The spec has validation errors that should be fixed.").red());
        }

        // Show preview
        println!("\n{}", style("═══════════════════════════════════════════════════").green());
        println!("{}", style("  Generated Specification Preview").green().bold());
        println!("{}\n", style("═══════════════════════════════════════════════════").green());

        let preview_lines: Vec<&str> = spec_text.lines().take(25).collect();
        for line in &preview_lines {
            println!("  {}", style(line).dim());
        }
        if spec_text.lines().count() > 25 {
            println!("  {}", style(format!("... ({} more lines)", spec_text.lines().count() - 25)).dim());
        }

        println!();

        // Action menu with refine option
        let mut options = vec![
            "✅ Accept and scaffold project",
            "✏️  Edit spec manually",
            "📄 Save spec to file and review later",
            "🔧 Refine idea - Improve current spec with additional instructions",
            "🔄 Regenerate with different idea",
            "❌ Cancel",
        ];

        // Add warning hint if not valid
        if !validation.is_valid {
            options[0] = "⚠️  Accept anyway (has errors)";
        }

        let action = Select::new()
            .with_prompt("What would you like to do?")
            .items(&options)
            .default(if validation.is_valid { 0 } else { 1 })
            .interact()?;

        match action {
            0 => {
                // Accept and scaffold
                if !validation.is_valid {
                    let confirm = Confirm::new()
                        .with_prompt("The spec has validation errors. Scaffold anyway?")
                        .default(false)
                        .interact()?;
                    if !confirm {
                        continue;
                    }
                }
                scaffold_with_spec_text(output_dir, &spec_text)?;
                println!("\n{}", style("✅ Project scaffolded successfully!").green().bold());
                break;
            }
            1 => {
                // Edit spec manually
                println!("{}", style("Opening editor... (save and close to continue)").dim());
                
                if let Some(edited) = Editor::new().edit(&spec_text)? {
                    let old_spec = spec_text.clone();
                    spec_text = edited;
                    
                    // Show diff
                    print_diff(&old_spec, &spec_text);
                    println!("{}", style("Spec updated. Re-validating...").cyan());
                } else {
                    println!("{}", style("No changes made.").dim());
                }
                // Continue loop to re-validate
            }
            2 => {
                // Save to file for review
                let spec_path = output_dir.join("app_spec.md");
                fs::write(&spec_path, &spec_text)?;
                println!("\n{} {}", 
                    style("📄 Spec saved to:").cyan(), 
                    style(spec_path.display()).green()
                );
                println!("{}", style("Review the file and run the scaffolder again with -s flag to use it.").dim());
                break;
            }
            3 => {
                // Refine idea - improve current spec with additional instructions
                println!("\n{}", style("─── Refine Specification ───").yellow().bold());
                println!("{}", style("Provide additional instructions to improve the current spec.").dim());
                
                let refinement: String = Input::new()
                    .with_prompt("Refinement instructions")
                    .interact_text()?;
                
                if refinement.trim().is_empty() {
                    println!("{}", style("No refinement instructions provided.").dim());
                    continue;
                }

                println!("\n{}", style("─────────────────────────────────────────────").dim());

                match refine_spec_from_idea(&spec_text, &refinement, model_owned.as_deref(), |msg| {
                    print!("{}", msg);
                    let _ = std::io::stdout().flush();
                }) {
                    Ok(refined) => {
                        spec_text = refined;
                        println!("\n{}", style("Specification refined. Re-validating...").cyan());
                    }
                    Err(e) => {
                        println!("\n{} {}", style("Refinement failed:").red().bold(), e);
                        println!("{}", style("Keeping the previous specification.").dim());
                    }
                }
                // Continue loop to re-validate
            }
            4 => {
                // Regenerate with different idea
                return run_generated_mode(output_dir, model_owned.as_deref());
            }
            _ => {
                println!("{}", style("Cancelled.").red());
                break;
            }
        }
    }

    Ok(())
}

/// Run the manual form-based spec mode (existing functionality)
fn run_manual_mode(output_dir: &Path) -> Result<()> {
    println!("\n{}", style("─── Manual Spec Creation ───").yellow().bold());

    // Project name
    let project_name: String = Input::new()
        .with_prompt("Project name")
        .interact_text()?;

    // Overview
    let overview: String = Input::new()
        .with_prompt("Brief project description")
        .interact_text()?;

    // Technology stack (optional)
    let include_tech = Confirm::new()
        .with_prompt("Define technology stack?")
        .default(true)
        .interact()?;

    let technology = if include_tech {
        Some(collect_tech_stack()?)
    } else {
        None
    };

    // Features
    println!("\n{}", style("Add Features").yellow().bold());
    println!("{}", style("(Enter features one at a time, empty to finish)").dim());
    
    let mut features = Vec::new();
    loop {
        let name: String = Input::new()
            .with_prompt("Feature name (empty to finish)")
            .allow_empty(true)
            .interact_text()?;

        if name.is_empty() {
            break;
        }

        let description: String = Input::new()
            .with_prompt("Feature description")
            .interact_text()?;

        let priority_idx = Select::new()
            .with_prompt("Priority")
            .items(["Critical", "High", "Medium", "Low"])
            .default(2)
            .interact()?;

        let priority = match priority_idx {
            0 => Priority::Critical,
            1 => Priority::High,
            2 => Priority::Medium,
            _ => Priority::Low,
        };

        features.push(Feature {
            name,
            description,
            priority,
            sub_features: Vec::new(),
        });

        println!("{} Feature added!", style("✓").green());
    }

    // Success criteria
    println!("\n{}", style("Success Criteria").yellow().bold());
    println!("{}", style("(Enter criteria one at a time, empty to finish)").dim());

    let mut success_criteria = Vec::new();
    loop {
        let criterion: String = Input::new()
            .with_prompt("Success criterion (empty to finish)")
            .allow_empty(true)
            .interact_text()?;

        if criterion.is_empty() {
            break;
        }

        success_criteria.push(criterion);
    }

    // Build the spec
    let spec = AppSpec {
        project_name,
        overview,
        features,
        success_criteria,
        technology,
        database: None,
        api_endpoints: None,
    };

    // Confirm and scaffold
    println!("\n{}", style("═══════════════════════════════════════════════════").cyan());
    println!("{}", style("  Summary").cyan().bold());
    println!("{}", style("═══════════════════════════════════════════════════").cyan());
    println!("  Project: {}", style(&spec.project_name).green());
    println!("  Features: {}", style(spec.features.len()).yellow());
    println!("  Criteria: {}", style(spec.success_criteria.len()).yellow());
    println!();

    let confirm = Confirm::new()
        .with_prompt("Generate plugin files?")
        .default(true)
        .interact()?;

    if confirm {
        scaffold_from_spec(output_dir, &spec)?;
    } else {
        println!("{}", style("Cancelled.").red());
    }

    Ok(())
}

fn collect_tech_stack() -> Result<TechStack> {
    // Languages
    let lang_options = vec![
        "Rust", "TypeScript", "JavaScript", "Python", "Go", "Other",
    ];
    let lang_indices = MultiSelect::new()
        .with_prompt("Languages (space to select, enter to confirm)")
        .items(&lang_options)
        .interact()?;

    let mut languages: Vec<String> = lang_indices
        .iter()
        .map(|&i| lang_options[i].to_string())
        .collect();

    // If "Other" selected, ask for custom
    if languages.contains(&"Other".to_string()) {
        languages.retain(|l| l != "Other");
        let custom: String = Input::new()
            .with_prompt("Other languages (comma-separated)")
            .allow_empty(true)
            .interact_text()?;
        for lang in custom.split(',') {
            let lang = lang.trim();
            if !lang.is_empty() {
                languages.push(lang.to_string());
            }
        }
    }

    // Frameworks
    let fw_options = vec![
        "React", "Next.js", "Vue", "Svelte", "Express", "Actix", "Axum", 
        "FastAPI", "Django", "Gin", "Other",
    ];
    let fw_indices = MultiSelect::new()
        .with_prompt("Frameworks (space to select, enter to confirm)")
        .items(&fw_options)
        .interact()?;

    let mut frameworks: Vec<String> = fw_indices
        .iter()
        .map(|&i| fw_options[i].to_string())
        .collect();

    // If "Other" selected, ask for custom
    if frameworks.contains(&"Other".to_string()) {
        frameworks.retain(|f| f != "Other");
        let custom: String = Input::new()
            .with_prompt("Other frameworks (comma-separated)")
            .allow_empty(true)
            .interact_text()?;
        for fw in custom.split(',') {
            let fw = fw.trim();
            if !fw.is_empty() {
                frameworks.push(fw.to_string());
            }
        }
    }

    Ok(TechStack {
        languages,
        frameworks,
        tools: Vec::new(),
    })
}
