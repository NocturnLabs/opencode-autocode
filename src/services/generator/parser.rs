use anyhow::{bail, Result};

/// Extract the project specification XML from OpenCode's output.
///
/// The output may contain various messages, tool call results, etc.
/// We need to find the XML specification block.
pub fn extract_spec_from_output(output: &str) -> Result<String> {
    // Look for the XML specification block
    // It should start with <project_specification> and end with </project_specification>

    if let Some(start) = output.find("<project_specification>") {
        if let Some(end) = output.find("</project_specification>") {
            let spec = &output[start..end + "</project_specification>".len()];
            return Ok(spec.to_string());
        }
    }

    // Try to find it in markdown code blocks
    if let Some(start) = output.find("```xml") {
        if let Some(end) = output[start..].find("```\n") {
            let block = &output[start + 6..start + end];
            if block.contains("<project_specification>") {
                return Ok(block.trim().to_string());
            }
        }
    }

    // If we can't find XML, maybe the output IS the spec (just wrapped differently)
    if output.contains("<project_name>") && output.contains("<overview>") {
        // Try to reconstruct from fragments
        bail!(
            "Could not extract complete specification. \
             The AI response may be malformed. Please try again."
        );
    }

    bail!(
        "No project specification found in OpenCode output. \
         The AI may have encountered an error or produced unexpected output.\n\n\
         Partial output:\n{}",
        output.chars().take(500).collect::<String>()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_spec_from_output() {
        let output = r#"
Some preamble text...

<project_specification>
<project_name>Test Project</project_name>
<overview>A test</overview>
</project_specification>

Some trailing text...
"#;

        let spec = extract_spec_from_output(output).unwrap();
        assert!(spec.starts_with("<project_specification>"));
        assert!(spec.ends_with("</project_specification>"));
        assert!(spec.contains("Test Project"));
    }

    #[test]
    fn test_extract_spec_no_match() {
        let output = "This is just random text without any spec";
        let result = extract_spec_from_output(output);
        assert!(result.is_err());
    }
}
