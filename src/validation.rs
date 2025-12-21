#![allow(dead_code)]
//! Output validation and diff generationty checks
//!
//! Validates generated project specifications for structural correctness
//! and quality metrics before scaffolding.

use anyhow::Result;
use console::style;
use quick_xml::events::Event;
use quick_xml::Reader;

/// Result of validating a spec
#[derive(Debug)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub stats: SpecStats,
}

/// Statistics about the spec
#[derive(Debug, Default)]
pub struct SpecStats {
    pub has_project_name: bool,
    pub has_overview: bool,
    pub has_tech_stack: bool,
    pub has_features: bool,
    pub has_database: bool,
    pub has_api_endpoints: bool,
    pub has_success_criteria: bool,
    pub feature_count: usize,
    pub endpoint_count: usize,
}

impl ValidationResult {
    /// Print the validation result in a user-friendly format
    pub fn print(&self) {
        if self.is_valid {
            println!("{}", style("✅ Spec validation passed").green().bold());
        } else {
            println!("{}", style("❌ Spec validation failed").red().bold());
        }

        if !self.errors.is_empty() {
            println!("\n{}", style("Errors:").red());
            for err in &self.errors {
                println!("   ❌ {}", err);
            }
        }

        if !self.warnings.is_empty() {
            println!("\n{}", style("Warnings:").yellow());
            for warn in &self.warnings {
                println!("   ⚠️  {}", warn);
            }
        }

        // Print stats
        println!("\n{}", style("Spec Statistics:").cyan());
        println!(
            "   Project Name: {}",
            bool_icon(self.stats.has_project_name)
        );
        println!("   Overview: {}", bool_icon(self.stats.has_overview));
        println!("   Tech Stack: {}", bool_icon(self.stats.has_tech_stack));
        println!(
            "   Features: {} ({})",
            bool_icon(self.stats.has_features),
            self.stats.feature_count
        );
        println!("   Database: {}", bool_icon(self.stats.has_database));
        println!(
            "   API Endpoints: {} ({})",
            bool_icon(self.stats.has_api_endpoints),
            self.stats.endpoint_count
        );
        println!(
            "   Success Criteria: {}",
            bool_icon(self.stats.has_success_criteria)
        );
    }
}

fn bool_icon(val: bool) -> &'static str {
    if val {
        "✓"
    } else {
        "✗"
    }
}

/// Validate a project specification
pub fn validate_spec(spec_text: &str) -> Result<ValidationResult> {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut stats = SpecStats::default();

    // Check if it looks like XML at all
    if !spec_text.contains("<project_specification>") {
        errors.push("Missing <project_specification> root element".to_string());
    }
    if !spec_text.contains("</project_specification>") {
        errors.push("Missing </project_specification> closing tag".to_string());
    }

    // Parse XML structure
    let mut reader = Reader::from_str(spec_text);
    reader.config_mut().trim_text(true);

    #[allow(unused_assignments)]
    let mut current_tag = String::new();
    let mut in_features = false;
    let mut in_endpoints = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                current_tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match current_tag.as_str() {
                    "project_name" => stats.has_project_name = true,
                    "overview" => stats.has_overview = true,
                    "technology_stack" => stats.has_tech_stack = true,
                    "core_features" => {
                        stats.has_features = true;
                        in_features = true;
                    }
                    "database_schema" | "database" => stats.has_database = true,
                    "api_endpoints" | "api_endpoints_summary" => {
                        stats.has_api_endpoints = true;
                        in_endpoints = true;
                    }
                    "success_criteria" => stats.has_success_criteria = true,
                    _ => {}
                }

                // Count features and endpoints
                if in_features && !["core_features", "feature"].contains(&current_tag.as_str()) {
                    // Any tag inside core_features that's not the container is a feature
                    if current_tag != "core_features" {
                        stats.feature_count += 1;
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if tag == "core_features" {
                    in_features = false;
                }
                if tag == "api_endpoints" || tag == "api_endpoints_summary" {
                    in_endpoints = false;
                }
            }
            Ok(Event::Text(e)) => {
                // Count API endpoint lines
                if in_endpoints {
                    let text = e.decode().unwrap_or_default();
                    for line in text.lines() {
                        if line.trim().starts_with("- ") && line.contains("/") {
                            stats.endpoint_count += 1;
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                errors.push(format!(
                    "XML parsing error at position {}: {}",
                    reader.buffer_position(),
                    e
                ));
                break;
            }
            _ => {}
        }
    }

    // Quality checks
    if !stats.has_project_name {
        errors.push("Missing <project_name> element".to_string());
    }
    if !stats.has_overview {
        errors.push("Missing <overview> element".to_string());
    }
    if !stats.has_features && !stats.has_tech_stack {
        warnings.push("Spec has no features or tech stack defined".to_string());
    }
    if stats.feature_count < 3 {
        warnings.push(format!(
            "Only {} features defined (recommend 3+)",
            stats.feature_count
        ));
    }
    if !stats.has_success_criteria {
        warnings.push("Missing success criteria - how will you know when it's done?".to_string());
    }

    let is_valid = errors.is_empty();

    Ok(ValidationResult {
        is_valid,
        errors,
        warnings,
        stats,
    })
}

/// Generate a diff between two spec versions
pub fn generate_diff(old_spec: &str, new_spec: &str) -> String {
    use similar::{ChangeTag, TextDiff};

    let diff = TextDiff::from_lines(old_spec, new_spec);
    let mut output = String::new();

    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            ChangeTag::Delete => "-",
            ChangeTag::Insert => "+",
            ChangeTag::Equal => " ",
        };
        output.push_str(&format!("{}{}", sign, change));
    }

    output
}

/// Print a colored diff
pub fn print_diff(old_spec: &str, new_spec: &str) {
    use similar::{ChangeTag, TextDiff};

    let diff = TextDiff::from_lines(old_spec, new_spec);

    println!("\n{}", style("─── Changes ───").cyan().bold());

    let mut has_changes = false;
    for change in diff.iter_all_changes() {
        match change.tag() {
            ChangeTag::Delete => {
                print!("{}", style(format!("-{}", change)).red());
                has_changes = true;
            }
            ChangeTag::Insert => {
                print!("{}", style(format!("+{}", change)).green());
                has_changes = true;
            }
            ChangeTag::Equal => {
                // Only show context lines, not the entire file
            }
        }
    }

    if !has_changes {
        println!("{}", style("   (no changes)").dim());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_good_spec() {
        let spec = r#"
<project_specification>
<project_name>Test Project</project_name>
<overview>A test project for validation</overview>
<technology_stack>
<frontend>React</frontend>
</technology_stack>
<core_features>
<feature1>Feature one</feature1>
<feature2>Feature two</feature2>
<feature3>Feature three</feature3>
</core_features>
<success_criteria>
- It works
</success_criteria>
</project_specification>
"#;
        let result = validate_spec(spec).unwrap();
        assert!(result.is_valid);
        assert!(result.stats.has_project_name);
        assert!(result.stats.has_overview);
        assert!(result.stats.feature_count >= 3);
    }

    #[test]
    fn test_validate_minimal_spec() {
        let spec = r#"
<project_specification>
<project_name>Minimal</project_name>
<overview>Just the basics</overview>
</project_specification>
"#;
        let result = validate_spec(spec).unwrap();
        assert!(result.is_valid); // Valid but has warnings
        assert!(!result.warnings.is_empty());
    }

    #[test]
    fn test_validate_broken_spec() {
        let spec = "This is not XML at all";
        let result = validate_spec(spec).unwrap();
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_generate_diff() {
        let old = "line1\nline2\nline3\n";
        let new = "line1\nmodified\nline3\n";
        let diff = generate_diff(old, new);
        assert!(diff.contains("-line2"));
        assert!(diff.contains("+modified"));
    }
}
