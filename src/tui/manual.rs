//! Manual mode - step-by-step spec creation

use anyhow::Result;
use std::path::Path;

use crate::scaffold::scaffold_from_spec;
use crate::spec::{AppSpec, Feature, Priority, TechStack};
use crate::tui::prompts::{confirm, input, multiline_input, print_error, print_success, select};

/// Run manual step-by-step spec creation
pub fn run_manual_mode(output_dir: &Path) -> Result<()> {
    println!("\n─── Manual Spec Creation ───");

    let spec = collect_spec_details()?;
    display_summary(&spec);

    if confirm("Generate plugin files?", true)? {
        scaffold_from_spec(output_dir, &spec)?;
    } else {
        print_error("Cancelled.");
    }

    Ok(())
}

fn collect_spec_details() -> Result<AppSpec> {
    let project_name = input("Project name", None)?;
    let overview = multiline_input("Brief description")?;

    let technology = if confirm("Define technology stack?", true)? {
        Some(collect_tech_stack()?)
    } else {
        None
    };

    let features = collect_features()?;
    let success_criteria = collect_success_criteria()?;

    let mut spec = AppSpec::new(&project_name);
    spec.overview = overview;
    spec.features = features;
    spec.success_criteria = success_criteria;
    spec.technology = technology;

    Ok(spec)
}

fn collect_features() -> Result<Vec<Feature>> {
    println!("\nAdd Features");
    println!("(Enter features one at a time, empty to finish)");

    let mut features = Vec::new();
    loop {
        let name = input("Feature name (empty to finish)", Some(""))?;

        if name.is_empty() {
            break;
        }

        let description = multiline_input("Description")?;
        let priority = select_priority()?;

        features.push(Feature {
            name,
            description,
            priority,
            sub_features: Vec::new(),
        });

        print_success("Feature added!");
    }
    Ok(features)
}

fn select_priority() -> Result<Priority> {
    let idx = select("Priority", &["Critical", "High", "Medium", "Low"], 2)?;

    Ok(match idx {
        0 => Priority::Critical,
        1 => Priority::High,
        2 => Priority::Medium,
        _ => Priority::Low,
    })
}

fn collect_success_criteria() -> Result<Vec<String>> {
    println!("\nSuccess Criteria");
    println!("(Enter criteria one at a time, empty to finish)");

    let mut criteria = Vec::new();
    loop {
        let criterion = input("Criterion (empty to finish)", Some(""))?;

        if criterion.is_empty() {
            break;
        }
        criteria.push(criterion);
    }
    Ok(criteria)
}

fn display_summary(spec: &AppSpec) {
    println!("\n═══════════════════════════════════════════════════");
    println!("  Summary");
    println!("═══════════════════════════════════════════════════");
    println!("  Project: {}", spec.project_name);
    println!("  Features: {}", spec.features.len());
    println!("  Criteria: {}", spec.success_criteria.len());
    println!();
}

fn collect_tech_stack() -> Result<TechStack> {
    let languages = collect_with_other(
        "Languages",
        &["Rust", "TypeScript", "JavaScript", "Python", "Go", "Other"],
        "Other languages (comma-separated)",
    )?;

    let frameworks = collect_with_other(
        "Frameworks",
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
    // Simplified: just prompt for comma-separated list
    println!("\n{} (pick from: {})", prompt, options.join(", "));
    let selection = input("Enter your choices (comma-separated)", Some(""))?;

    let mut items: Vec<String> = selection
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if items.iter().any(|s| s.eq_ignore_ascii_case("other")) {
        items.retain(|s| !s.eq_ignore_ascii_case("other"));
        let custom = input(other_prompt, Some(""))?;
        for item in custom.split(',') {
            let item = item.trim();
            if !item.is_empty() {
                items.push(item.to_string());
            }
        }
    }

    Ok(items)
}
