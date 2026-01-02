//! Manual mode - step-by-step spec creation

use anyhow::Result;
use console::style;
use dialoguer::{Confirm, Input, MultiSelect, Select};
use std::path::Path;

use crate::scaffold::scaffold_from_spec;
use crate::spec::{AppSpec, Feature, Priority, TechStack};
use crate::tui::inputs::read_multiline;

/// Run manual step-by-step spec creation
pub fn run_manual_mode(output_dir: &Path) -> Result<()> {
    println!(
        "\n{}",
        style("─── Manual Spec Creation ───").yellow().bold()
    );

    let spec = collect_spec_details()?;
    display_summary(&spec);

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

fn collect_spec_details() -> Result<AppSpec> {
    let project_name: String = Input::new().with_prompt("Project name").interact_text()?;
    let overview: String = read_multiline("Brief description")?;

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

        let description: String = read_multiline("Description")?;
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

fn display_summary(spec: &AppSpec) {
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
