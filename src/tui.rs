//! Interactive TUI for building app spec

use anyhow::Result;
use console::style;
use dialoguer::{Confirm, Input, MultiSelect, Select};
use std::path::Path;

use crate::scaffold::scaffold_from_spec;
use crate::spec::{AppSpec, Feature, Priority, TechStack};

/// Run the interactive TUI to build an app spec
pub fn run_interactive(output_dir: &Path) -> Result<()> {
    println!("\n{}", style("═══════════════════════════════════════════════════").cyan());
    println!("{}", style("  OpenCode Autonomous Plugin - Interactive Setup").cyan().bold());
    println!("{}\n", style("═══════════════════════════════════════════════════").cyan());

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
            .items(&["Critical", "High", "Medium", "Low"])
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
