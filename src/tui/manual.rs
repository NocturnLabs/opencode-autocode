//! Manual mode - step-by-step spec creation

use anyhow::Result;
use std::path::Path;

use crate::config::Config;
use crate::services::scaffold::scaffold_from_spec;
use crate::theming::{accent, highlight, muted, primary, symbols};

use crate::spec::{AppSpec, Feature, Priority, TechStack};
use crate::tui::prompts::{confirm, input, multiline_input, print_error, print_success, select};

/// Run manual step-by-step spec creation
pub fn run_manual_mode(output_dir: &Path, config: &Config) -> Result<()> {
    println!(
        "\n{} {}",
        accent(symbols::CHEVRON),
        highlight("Manual Spec Creation")
    );

    let spec = collect_spec_details(config)?;
    display_summary(&spec);

    if confirm("Generate plugin files?", true)? {
        scaffold_from_spec(output_dir, &spec)?;
    } else {
        print_error("Cancelled.");
    }

    Ok(())
}

fn collect_spec_details(config: &Config) -> Result<AppSpec> {
    let project_name = input("Project name", None)?;
    let overview = multiline_input("Brief description")?;

    let technology = if confirm("Define technology stack?", true)? {
        Some(collect_tech_stack()?)
    } else {
        None
    };

    let features = collect_features(config)?;
    let success_criteria = collect_success_criteria()?;

    let mut spec = AppSpec::new(&project_name);
    spec.overview = overview;
    spec.features = features;
    spec.success_criteria = success_criteria;
    spec.technology = technology;

    Ok(spec)
}

fn collect_features(config: &Config) -> Result<Vec<Feature>> {
    println!(
        "\n{} {}",
        accent(symbols::CHEVRON),
        highlight("Add Features")
    );
    println!(
        "{}",
        muted("Enter features one at a time, empty to finish.")
    );

    let mut features = Vec::new();
    loop {
        let name = input("Feature name (empty to finish)", Some(""))?;

        if name.is_empty() {
            break;
        }

        let description = multiline_input("Description")?;
        let priority = select_priority(config)?;

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

/// @param config Loaded configuration.
/// @returns The selected priority value.
fn select_priority(config: &Config) -> Result<Priority> {
    let options = build_priority_options(config);
    let labels: Vec<&str> = options.iter().map(|opt| opt.label.as_str()).collect();
    let default_index = options
        .iter()
        .position(|opt| opt.value == Priority::Medium)
        .unwrap_or(0);

    let idx = select("Priority", &labels, default_index)?;

    Ok(options
        .get(idx)
        .map(|opt| opt.value.clone())
        .unwrap_or(Priority::Medium))
}

/// Priority option metadata for manual selection.
struct PriorityOption {
    /// Label shown to the user.
    label: String,
    /// Priority enum value.
    value: Priority,
}

/// @param config Loaded configuration.
/// @returns Priority options in configured order.
fn build_priority_options(config: &Config) -> Vec<PriorityOption> {
    let mut options = Vec::new();
    for entry in &config.features.priorities {
        match entry.to_lowercase().as_str() {
            "critical" => options.push(PriorityOption {
                label: "Critical".to_string(),
                value: Priority::Critical,
            }),
            "high" => options.push(PriorityOption {
                label: "High".to_string(),
                value: Priority::High,
            }),
            "medium" => options.push(PriorityOption {
                label: "Medium".to_string(),
                value: Priority::Medium,
            }),
            "low" => options.push(PriorityOption {
                label: "Low".to_string(),
                value: Priority::Low,
            }),
            _ => {}
        }
    }

    if options.is_empty() {
        options = vec![
            PriorityOption {
                label: "Critical".to_string(),
                value: Priority::Critical,
            },
            PriorityOption {
                label: "High".to_string(),
                value: Priority::High,
            },
            PriorityOption {
                label: "Medium".to_string(),
                value: Priority::Medium,
            },
            PriorityOption {
                label: "Low".to_string(),
                value: Priority::Low,
            },
        ];
    }

    options
}

fn collect_success_criteria() -> Result<Vec<String>> {
    println!(
        "\n{} {}",
        accent(symbols::CHEVRON),
        highlight("Success Criteria")
    );
    println!(
        "{}",
        muted("Enter criteria one at a time, empty to finish.")
    );

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
    println!("\n{} {}", accent(symbols::CHEVRON), highlight("Summary"));
    println!("{} {}", muted("Project:"), primary(&spec.project_name));
    println!("{} {}", muted("Features:"), primary(spec.features.len()));
    println!(
        "{} {}",
        muted("Criteria:"),
        primary(spec.success_criteria.len())
    );
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
    println!(
        "\n{} {}",
        accent(symbols::CHEVRON),
        highlight(format!("{} (pick from: {})", prompt, options.join(", ")))
    );
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
