use crate::config::Config;
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

    let guidance = if config.generation.complexity == "minimal" {
        "The target is a minimal, lightweight implementation. Focus only on the absolute core."
    } else {
        "The target is a comprehensive, production-ready specification with deep detail."
    };

    let generator_prompt = template_xml::render_template(GENERATOR_PROMPT)
        .unwrap_or_else(|_| GENERATOR_PROMPT.to_string());

    generator_prompt
        .replace("{{IDEA}}", idea)
        .replace("{{TESTING_PREFERENCE}}", &pref_text)
        .replace("{{COMPLEXITY_GUIDANCE}}", guidance)
}

/// Build the subagent-based generation prompt by inserting the user's idea and constraints.
pub fn build_subagent_prompt(
    idea: &str,
    testing_preference: Option<&str>,
    _config: &Config,
) -> String {
    let pref_text = match testing_preference {
        Some(pref) if !pref.trim().is_empty() => format!(
            "\n**User Preference:** QA/Testing framework should be: {}\n",
            pref
        ),
        _ => String::new(),
    };

    let subagent_prompt = template_xml::render_template(SUBAGENT_PROMPT)
        .unwrap_or_else(|_| SUBAGENT_PROMPT.to_string());

    subagent_prompt
        .replace("{{IDEA}}", idea)
        .replace("{{TESTING_PREFERENCE}}", &pref_text)
        .replace("{{BLUEPRINT}}", "[The blueprint you generated above]")
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
