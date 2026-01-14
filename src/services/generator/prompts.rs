use crate::config::{ComplexityLevel, Config, GenerationRequirements};
use crate::template_xml;

/// Embedded prompt template for spec generation (legacy single-pass)
const GENERATOR_PROMPT: &str = include_str!("../../../templates/generator_prompt.xml");

/// Embedded prompt template for subagent-based parallel generation
const SUBAGENT_PROMPT: &str = include_str!("../../../templates/generator/subagent_prompt.xml");

/// Embedded prompt template for spec refinement
const REFINE_PROMPT: &str = include_str!("../../../templates/refine_prompt.xml");

/// Embedded prompt template for fixing malformed XML
const FIX_MALFORMED_PROMPT: &str =
    include_str!("../../../templates/generator/fix_malformed_xml.xml");

/// @param requirements Minimum content requirements to enforce.
/// @returns A prompt-ready summary of minimum requirements.
fn build_requirements_section(requirements: GenerationRequirements) -> String {
    format!(
        "## Minimum Scope Requirements\n\n- Minimum features: {}\n- Minimum database tables: {}\n- Minimum API endpoints: {}\n- Minimum implementation steps: {}",
        requirements.min_features,
        requirements.min_database_tables,
        requirements.min_api_endpoints,
        requirements.min_implementation_steps
    )
}

/// @param enabled Whether the section should be included.
/// @param content The XML example content to inject.
/// @returns The section string if enabled, otherwise an empty string.
fn build_section_block(enabled: bool, content: &str) -> String {
    if enabled {
        format!("{}\n\n", content.trim_end())
    } else {
        String::new()
    }
}

/// @param indent Prefix indentation for multi-line sections.
/// @returns Example testing strategy block.
fn build_testing_strategy_block(indent: &str) -> String {
    format!(
        "{indent}<testing_strategy>\n{indent}  <unit_tests>Coverage targets and priority areas</unit_tests>\n{indent}  <integration_tests>Integration test approach</integration_tests>\n{indent}  <e2e_tests>\n{indent}   - Critical flows to cover\n{indent}   - **MANDATORY**: Every core feature MUST have a scriptable E2E test\n{indent}   - verification_command MUST invoke E2E tests\n{indent}  </e2e_tests>\n{indent}  <entry_point_verification>\n{indent}   - The main entry point MUST be wired to call all handlers.\n{indent}   - verification_command MUST check that the application RUNS.\n{indent}  </entry_point_verification>\n{indent}</testing_strategy>"
    )
}

/// @param indent Prefix indentation for the line.
/// @param tag XML tag name.
/// @param content Example content for the tag.
/// @returns Example XML block for a single-line section.
fn build_single_line_section(indent: &str, tag: &str, content: &str) -> String {
    format!("{indent}<{tag}>{content}</{tag}>")
}

/// @param prompt The prompt template with placeholders.
/// @param config The generation configuration.
/// @param indent Prefix indentation for inserted XML blocks.
/// @returns The prompt with optional output sections filled in.
fn apply_output_sections(prompt: String, config: &Config, indent: &str) -> String {
    let security_block = build_section_block(
        config.generation.include_security_section,
        &build_single_line_section(indent, "security", "Security requirements and measures"),
    );
    let testing_block = build_section_block(
        config.generation.include_testing_strategy,
        &build_testing_strategy_block(indent),
    );
    let devops_block = build_section_block(
        config.generation.include_devops_section,
        &build_single_line_section(indent, "devops", "Deployment and operations plan"),
    );
    let accessibility_block = build_section_block(
        config.generation.include_accessibility,
        &build_single_line_section(
            indent,
            "accessibility",
            "Accessibility requirements and testing",
        ),
    );
    let future_block = build_section_block(
        config.generation.include_future_enhancements,
        &build_single_line_section(
            indent,
            "future_enhancements",
            "Potential post-launch features",
        ),
    );

    prompt
        .replace("{{SECURITY_SECTION}}", security_block.trim_end())
        .replace("{{TESTING_STRATEGY_SECTION}}", testing_block.trim_end())
        .replace("{{DEVOPS_SECTION}}", devops_block.trim_end())
        .replace("{{ACCESSIBILITY_SECTION}}", accessibility_block.trim_end())
        .replace("{{FUTURE_ENHANCEMENTS_SECTION}}", future_block.trim_end())
}

/// @param idea The user-provided idea description.
/// @param config Generation config used for section requirements.
/// @returns Task instruction for the quality subagent.
fn build_quality_task(idea: &str, config: &Config) -> String {
    let mut sections = vec!["implementation steps", "success criteria"];

    if config.generation.include_security_section {
        sections.push("security");
    }
    if config.generation.include_testing_strategy {
        sections.push("testing strategy");
    }
    if config.generation.include_devops_section {
        sections.push("devops");
    }
    if config.generation.include_accessibility {
        sections.push("accessibility");
    }
    if config.generation.include_future_enhancements {
        sections.push("future enhancements");
    }

    format!("Generate {} for: {}", sections.join(", "), idea)
}

/// @param requirements Minimum requirements for core features.
/// @returns Guidance for feature count targets.
fn build_feature_count_guidance(requirements: GenerationRequirements) -> String {
    format!(
        "IMPORTANT: Generate at least {} features depending on project complexity. Each feature must include:\n- Name\n- Detailed description (2-4 sentences)\n- Sub-features (list of specific capabilities)\n- Error handling approach\n- Edge cases to consider",
        requirements.min_features
    )
}

/// @param requirements Minimum requirements for schemas and endpoints.
/// @returns Guidance for table and endpoint counts.
fn build_architecture_count_guidance(requirements: GenerationRequirements) -> String {
    format!(
        "IMPORTANT:\n- Database: Define at least {} tables with full column specifications, types, constraints, and relationships\n- API Endpoints: Define at least {} endpoints in structured <endpoint> elements with method, path, and description",
        requirements.min_database_tables, requirements.min_api_endpoints
    )
}

/// @param requirements Minimum requirements for implementation steps.
/// @returns Guidance for implementation phase count.
fn build_quality_count_guidance(requirements: GenerationRequirements) -> String {
    format!(
        "IMPORTANT: Implementation steps should have at least {} phases with specific tasks and verification criteria.",
        requirements.min_implementation_steps
    )
}

/// Build the generation prompt by inserting the user's idea and configuration constraints.
pub fn build_generation_prompt(
    idea: &str,
    testing_preference: Option<&str>,
    config: &Config,
) -> String {
    let pref_text = match testing_preference {
        Some(pref) if !pref.trim().is_empty() => format!(
            "\n## User Preferences\n\nTesting & QA Framework Preference: {}\n",
            pref
        ),
        _ => String::new(),
    };

    let guidance = if config.generation.complexity == ComplexityLevel::Minimal {
        "The target is a minimal, lightweight implementation. Focus only on the absolute core."
    } else {
        "The target is a comprehensive, production-ready specification with deep detail."
    };

    let requirements = config.generation.requirements();
    let requirements_section = build_requirements_section(requirements);

    let mut generator_prompt = template_xml::render_template(GENERATOR_PROMPT)
        .unwrap_or_else(|_| GENERATOR_PROMPT.to_string());

    generator_prompt = generator_prompt.replace(
        "## Output Format",
        &format!("{}\n\n## Output Format", requirements_section),
    );

    let generator_prompt = generator_prompt
        .replace("{{IDEA}}", idea)
        .replace("{{TESTING_PREFERENCE}}", &pref_text)
        .replace("{{COMPLEXITY_GUIDANCE}}", guidance);

    apply_output_sections(generator_prompt, config, "")
}

/// Build the subagent-based generation prompt by inserting the user's idea and constraints.
pub fn build_subagent_prompt(
    idea: &str,
    testing_preference: Option<&str>,
    config: &Config,
) -> String {
    let pref_text = match testing_preference {
        Some(pref) if !pref.trim().is_empty() => format!(
            "\n**User Preference:** QA/Testing framework should be: {}\n",
            pref
        ),
        _ => String::new(),
    };

    let requirements = config.generation.requirements();

    let subagent_prompt = template_xml::render_template(SUBAGENT_PROMPT)
        .unwrap_or_else(|_| SUBAGENT_PROMPT.to_string());

    let subagent_prompt = subagent_prompt
        .replace("{{IDEA}}", idea)
        .replace("{{TESTING_PREFERENCE}}", &pref_text)
        .replace("{{BLUEPRINT}}", "[The blueprint you generated above]")
        .replace(
            "{{FEATURE_COUNT_GUIDANCE}}",
            &build_feature_count_guidance(requirements),
        )
        .replace(
            "{{ARCHITECTURE_COUNT_GUIDANCE}}",
            &build_architecture_count_guidance(requirements),
        )
        .replace("{{QUALITY_TASK}}", &build_quality_task(idea, config))
        .replace(
            "{{QUALITY_COUNT_GUIDANCE}}",
            &build_quality_count_guidance(requirements),
        );

    apply_output_sections(subagent_prompt, config, "  ")
}

/// Build the refinement prompt by inserting the current spec and refinement instructions.
pub fn build_refine_prompt(current_spec: &str, refinement: &str) -> String {
    let refine_prompt =
        template_xml::render_template(REFINE_PROMPT).unwrap_or_else(|_| REFINE_PROMPT.to_string());

    refine_prompt
        .replace("{{EXISTING_SPEC}}", current_spec)
        .replace("{{REFINEMENT}}", refinement)
}

/// Build the fix prompt by inserting the original idea and error message.
pub fn build_fix_prompt(idea: &str, errors: &str, partial_output: Option<&str>) -> String {
    let fix_prompt = template_xml::render_template(FIX_MALFORMED_PROMPT)
        .unwrap_or_else(|_| FIX_MALFORMED_PROMPT.to_string())
        .replace("{{IDEA}}", idea)
        .replace("{{ERRORS}}", errors);

    let partial_text = match partial_output {
        Some(output) if !output.trim().is_empty() => {
            let truncated = if output.len() > 10000 {
                format!("... (truncated) ...\n{}", &output[output.len() - 10000..])
            } else {
                output.to_string()
            };
            format!(
                "\n## Partial Output (for context)\n\n```\n{}\n```\n",
                truncated
            )
        }
        _ => String::new(),
    };

    fix_prompt.replace("{{PARTIAL_OUTPUT}}", &partial_text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_generation_prompt() {
        let idea = "A todo app with tags";
        let config = Config::default();
        let prompt = build_generation_prompt(idea, None, &config);

        assert!(prompt.contains("A todo app with tags"));
        assert!(prompt.contains("<project_specification>"));
        assert!(!prompt.contains("{{IDEA}}"));
    }

    #[test]
    fn test_build_generation_prompt_contains_constraints() {
        let idea = "A complex ERP";
        let config = Config::default();
        let prompt = build_generation_prompt(idea, None, &config);

        assert!(prompt.contains("production-ready specification with deep detail"));
    }
}
