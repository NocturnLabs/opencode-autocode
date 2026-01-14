//! Output validation and diff generation
//!
//! Validates generated project specifications for structural correctness
//! and quality metrics before scaffolding.
//!
//! # Features
//!
//! - XML structure validation
//! - Content quality checks
//! - Configuration-aware validation rules
//! - Statistical analysis of specifications
//! - Diff generation for specification changes

use anyhow::Result;
use quick_xml::events::Event;
use quick_xml::Reader;

use crate::config::{Config, GenerationRequirements};

/// Result of validating a spec
#[derive(Debug, Clone, Default)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub stats: SpecStats,
}

/// Statistics about the spec
#[derive(Debug, Clone, Default)]
pub struct SpecStats {
    pub has_project_name: bool,
    pub has_overview: bool,
    pub has_tech_stack: bool,
    pub has_features: bool,
    pub has_database: bool,
    pub has_api_endpoints: bool,
    pub has_success_criteria: bool,
    pub has_security: bool,
    pub has_testing_strategy: bool,
    pub has_devops: bool,
    pub has_accessibility: bool,
    pub has_future_enhancements: bool,
    pub feature_count: usize,
    pub endpoint_count: usize,
    pub database_table_count: usize,
    pub implementation_step_count: usize,
}

impl ValidationResult {
    /// Print the validation result in a user-friendly format
    ///
    /// Displays validation status, errors, warnings, and detailed statistics
    /// about the specification content.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opencode_forger::validation::validate_spec;
    /// let spec_text = "<project_specification><project_name>Test</project_name><overview>Overview</overview></project_specification>";
    /// let result = validate_spec(&spec_text).unwrap();
    /// result.print();
    /// ```
    pub fn print(&self) {
        if self.is_valid {
            println!("✅ Spec validation passed");
        } else {
            println!("❌ Spec validation failed");
        }

        if !self.errors.is_empty() {
            println!("\nErrors:");
            for err in &self.errors {
                println!("   ❌ {}", err);
            }
        }

        if !self.warnings.is_empty() {
            println!("\nWarnings:");
            for warn in &self.warnings {
                println!("   ⚠️  {}", warn);
            }
        }

        // Print stats
        println!("\nSpec Statistics:");
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
        println!(
            "   Database: {} ({})",
            bool_icon(self.stats.has_database),
            self.stats.database_table_count
        );
        println!(
            "   API Endpoints: {} ({})",
            bool_icon(self.stats.has_api_endpoints),
            self.stats.endpoint_count
        );
        println!(
            "   Implementation Steps: {} ({})",
            bool_icon(self.stats.implementation_step_count > 0),
            self.stats.implementation_step_count
        );
        println!("   Security: {}", bool_icon(self.stats.has_security));
        println!(
            "   Testing Strategy: {}",
            bool_icon(self.stats.has_testing_strategy)
        );
        println!("   DevOps: {}", bool_icon(self.stats.has_devops));
        println!(
            "   Accessibility: {}",
            bool_icon(self.stats.has_accessibility)
        );
        println!(
            "   Future Enhancements: {}",
            bool_icon(self.stats.has_future_enhancements)
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

/// Validation rules derived from configuration.
#[derive(Debug, Clone, Copy)]
pub struct ValidationRules {
    pub requirements: GenerationRequirements,
    pub include_security_section: bool,
    pub include_testing_strategy: bool,
    pub include_devops_section: bool,
    pub include_accessibility: bool,
    pub include_future_enhancements: bool,
}

impl ValidationRules {
    /// Create validation rules from configuration
    ///
    /// Converts generation configuration into validation rules that determine
    /// what content is required and what quality thresholds must be met.
    ///
    /// # Arguments
    ///
    /// * `config` - Loaded configuration containing generation settings
    ///
    /// # Returns
    ///
    /// ValidationRules instance configured based on the provided config
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opencode_forger::config::Config;
    /// use opencode_forger::validation::ValidationRules;
    /// let config = Config::default();
    /// let rules = ValidationRules::from_config(&config);
    /// ```
    pub fn from_config(config: &Config) -> Self {
        Self {
            requirements: config.generation.requirements(),
            include_security_section: config.generation.include_security_section,
            include_testing_strategy: config.generation.include_testing_strategy,
            include_devops_section: config.generation.include_devops_section,
            include_accessibility: config.generation.include_accessibility,
            include_future_enhancements: config.generation.include_future_enhancements,
        }
    }
}

/// Strip XML comments from text to prevent bypass attacks.
/// Comments like <!-- hidden content --> are removed before validation.
///
/// This prevents malicious or malformed content from being hidden in XML comments
/// that could bypass validation checks.
///
/// # Arguments
///
/// * `text` - XML text that may contain comments
///
/// # Returns
///
/// String with XML comments removed
///
/// # Examples
///
/// ```rust,ignore
/// let clean_text = strip_xml_comments("<root><!-- bad --><item/></root>");
/// assert_eq!(clean_text, "<root><item/></root>");
/// ```
fn strip_xml_comments(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '<' {
            // Check for comment start
            let mut lookahead = String::new();
            lookahead.push(c);

            for _ in 0..3 {
                if let Some(&next) = chars.peek() {
                    lookahead.push(next);
                    chars.next();
                }
            }

            if lookahead == "<!--" {
                // Skip until we find -->
                let mut end_seq = String::new();
                for ch in chars.by_ref() {
                    end_seq.push(ch);
                    if end_seq.len() > 3 {
                        end_seq.remove(0);
                    }
                    if end_seq == "-->" {
                        break;
                    }
                }
            } else {
                // Not a comment, include the lookahead
                result.push_str(&lookahead);
            }
        } else {
            result.push(c);
        }
    }

    result
}

/// Validate a project specification.
///
/// Performs basic validation of a project specification without configuration-specific rules.
///
/// # Arguments
///
/// * `spec_text` - XML specification text to validate
///
/// # Returns
///
/// Result containing ValidationResult with validation status and details
///
/// # Examples
///
/// ```rust
/// use opencode_forger::validation::validate_spec;
/// let spec = "<project_specification><project_name>Test</project_name><overview>Overview</overview></project_specification>";
/// let result = validate_spec(spec).unwrap();
/// if result.is_valid {
///     println!("Spec is valid!");
/// }
/// ```
pub fn validate_spec(spec_text: &str) -> Result<ValidationResult> {
    validate_spec_with_rules(spec_text, None)
}

/// Validate a project specification with configuration-aware rules.
///
/// Performs validation using rules derived from the provided configuration,
/// including minimum requirements for features, database tables, API endpoints, etc.
///
/// # Arguments
///
/// * `spec_text` - XML specification text to validate
/// * `config` - Configuration containing validation rules
///
/// # Returns
///
/// Result containing ValidationResult with validation status and details
///
/// # Examples
///
/// ```rust
/// use opencode_forger::config::Config;
/// use opencode_forger::validation::validate_spec_with_config;
/// let spec = "<project_specification><project_name>Test</project_name><overview>Overview</overview></project_specification>";
/// let config = Config::default();
/// let result = validate_spec_with_config(spec, &config).unwrap();
/// ```
pub fn validate_spec_with_config(spec_text: &str, config: &Config) -> Result<ValidationResult> {
    validate_spec_with_rules(spec_text, Some(ValidationRules::from_config(config)))
}

/// Internal function to validate specification with optional rules.
///
/// Core validation logic that handles both basic and configuration-aware validation.
///
/// # Arguments
///
/// * `spec_text` - XML specification text to validate
/// * `rules` - Optional validation rules for configuration-aware checks
///
/// # Returns
///
/// Result containing ValidationResult with validation status and details
fn validate_spec_with_rules(
    spec_text: &str,
    rules: Option<ValidationRules>,
) -> Result<ValidationResult> {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut stats = SpecStats::default();

    // Strip XML comments before validation to prevent bypass attacks
    // where malformed content is hidden inside <!-- ... --> blocks
    let stripped_spec = strip_xml_comments(spec_text);

    // Check if it looks like XML at all (using stripped version)
    if !stripped_spec.contains("<project_specification>") {
        errors.push("Missing <project_specification> root element".to_string());
    }
    if !stripped_spec.contains("</project_specification>") {
        errors.push("Missing </project_specification> closing tag".to_string());
    }

    // Parse XML structure
    let mut reader = Reader::from_str(spec_text);
    reader.config_mut().trim_text(true);

    #[allow(unused_assignments)]
    let mut current_tag = String::new();
    let mut in_features = false;
    let mut in_endpoints = false;
    let mut in_tables = false;
    let mut in_steps = false;

    let mut feature_tags_found = 0;
    let mut other_tags_in_features = 0;
    let mut table_tags_found = 0;
    let mut table_lines_found = 0;

    let mut step_tags_found = 0;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
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
                    "tables" => {
                        stats.has_database = true;
                        in_tables = true;
                    }
                    "api_endpoints" | "api_endpoints_summary" => {
                        stats.has_api_endpoints = true;
                        in_endpoints = true;
                    }
                    "implementation_steps" => in_steps = true,
                    "success_criteria" => stats.has_success_criteria = true,
                    "security" => stats.has_security = true,
                    "testing_strategy" => stats.has_testing_strategy = true,
                    "devops" => stats.has_devops = true,
                    "accessibility" => stats.has_accessibility = true,
                    "future_enhancements" => stats.has_future_enhancements = true,
                    "feature" => {
                        if in_features {
                            feature_tags_found += 1;
                        }
                    }
                    "endpoint" => {
                        if in_endpoints {
                            stats.endpoint_count += 1;
                        }
                    }
                    "step" => {
                        if in_steps {
                            step_tags_found += 1;
                        }
                    }
                    "table" => {
                        if in_tables {
                            table_tags_found += 1;
                        }
                    }
                    "column" => {
                        if in_tables {
                            stats.has_database = true;
                        }
                    }
                    _ => {
                        if in_features && current_tag != "core_features" {
                            other_tags_in_features += 1;
                        }
                        if in_tables && current_tag != "tables" && current_tag != "column" {
                            table_tags_found += 1;
                        }
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
                if tag == "tables" {
                    in_tables = false;
                }
                if tag == "implementation_steps" {
                    in_steps = false;
                }
            }
            Ok(Event::Text(e)) => {
                // Count API endpoint lines (fallback for summary mode)
                if in_endpoints {
                    let text = e.decode().unwrap_or_default();
                    for line in text.lines() {
                        let trimmed = line.trim();
                        if trimmed.starts_with("- ")
                            && (trimmed.contains("/") || trimmed.contains("METHOD"))
                        {
                            stats.endpoint_count += 1;
                        }
                    }
                }

                if in_tables {
                    let text = e.decode().unwrap_or_default();
                    for line in text.lines() {
                        let trimmed = line.trim();
                        if trimmed.starts_with('-') {
                            table_lines_found += 1;
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

    stats.implementation_step_count = step_tags_found;
    stats.database_table_count = if table_tags_found > 0 {
        table_tags_found
    } else {
        table_lines_found
    };

    // Determine final feature count: prefer structured <feature> tags if present
    stats.feature_count = if feature_tags_found > 0 {
        feature_tags_found
    } else {
        other_tags_in_features
    };

    if let Some(rules) = rules {
        let requirements = rules.requirements;
        if stats.feature_count < requirements.min_features as usize {
            errors.push(format!(
                "Spec has {} feature(s), requires at least {}",
                stats.feature_count, requirements.min_features
            ));
        }
        if stats.database_table_count < requirements.min_database_tables as usize {
            errors.push(format!(
                "Spec has {} database table(s), requires at least {}",
                stats.database_table_count, requirements.min_database_tables
            ));
        }
        if stats.endpoint_count < requirements.min_api_endpoints as usize {
            errors.push(format!(
                "Spec has {} API endpoint(s), requires at least {}",
                stats.endpoint_count, requirements.min_api_endpoints
            ));
        }
        if stats.implementation_step_count < requirements.min_implementation_steps as usize {
            errors.push(format!(
                "Spec has {} implementation step(s), requires at least {}",
                stats.implementation_step_count, requirements.min_implementation_steps
            ));
        }
        if rules.include_security_section && !stats.has_security {
            errors.push("Missing <security> section in spec".to_string());
        }
        if rules.include_testing_strategy && !stats.has_testing_strategy {
            errors.push("Missing <testing_strategy> section in spec".to_string());
        }
        if rules.include_devops_section && !stats.has_devops {
            errors.push("Missing <devops> section in spec".to_string());
        }
        if rules.include_accessibility && !stats.has_accessibility {
            errors.push("Missing <accessibility> section in spec".to_string());
        }
        if rules.include_future_enhancements && !stats.has_future_enhancements {
            errors.push("Missing <future_enhancements> section in spec".to_string());
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

/// Print a colored diff
///
/// Displays the differences between two specification versions using colored output.
/// Shows additions (+) and deletions (-) between the old and new specification text.
///
/// # Arguments
///
/// * `old_spec` - Original specification text
/// * `new_spec` - Modified specification text
///
/// # Examples
///
/// ```rust
/// use opencode_forger::validation::print_diff;
/// let old_spec = "<project>...</project>";
/// let new_spec = "<project>...</project>";
/// print_diff(old_spec, new_spec);
/// ```
pub fn print_diff(old_spec: &str, new_spec: &str) {
    use similar::{ChangeTag, TextDiff};

    let diff = TextDiff::from_lines(old_spec, new_spec);

    println!("\n─── Changes ───");

    let mut has_changes = false;
    for change in diff.iter_all_changes() {
        match change.tag() {
            ChangeTag::Delete => {
                print!("-{}", change);
                has_changes = true;
            }
            ChangeTag::Insert => {
                print!("+{}", change);
                has_changes = true;
            }
            ChangeTag::Equal => {
                // Only show context lines, not the entire file
            }
        }
    }

    if !has_changes {
        println!("   (no changes)");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_structured_features() {
        let spec = r#"
<project_specification>
<project_name>Structured</project_name>
<overview>A test with structured features</overview>
<core_features>
<feature>
  <name>Feature 1</name>
</feature>
<feature>
  <name>Feature 2</name>
</feature>
<feature>
  <name>Feature 3</name>
</feature>
</core_features>
</project_specification>
"#;
        let result = validate_spec(spec).unwrap();
        assert!(result.is_valid);
        assert_eq!(result.stats.feature_count, 3);
    }

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
}
