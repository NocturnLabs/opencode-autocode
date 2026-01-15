//! XML sanitization for AI-generated project specifications.
//!
//! LLMs often produce XML with common issues:
//! - Unescaped `&` (e.g., "R&D" instead of "R&amp;D")
//! - Unescaped `<` in text (e.g., "<100ms" instead of "&lt;100ms")
//! - Mismatched closing tags (e.g., `</api>` when `<api_style>` was opened)
//!
//! This module provides a sanitizer that fixes these issues before XML parsing.

/// Sanitize AI-generated XML specification text.
///
/// Applies the following fixes:
/// 1. Escapes stray `&` characters that aren't valid entity references
/// 2. Escapes `<` characters that appear before digits or whitespace
///
/// # Arguments
///
/// * `spec` - The raw XML specification text from the AI
///
/// # Returns
///
/// The sanitized XML string that should parse without entity/character errors.
///
/// # Examples
///
/// ```rust
/// use opencode_forger::services::generator::sanitize::sanitize_spec_xml;
///
/// let raw = "<overview>R&D team achieved <100ms latency</overview>";
/// let clean = sanitize_spec_xml(raw);
/// assert_eq!(clean, "<overview>R&amp;D team achieved &lt;100ms latency</overview>");
/// ```
pub fn sanitize_spec_xml(spec: &str) -> String {
    let mut result = String::with_capacity(spec.len() + 100);
    let chars: Vec<char> = spec.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let c = chars[i];

        if c == '&' {
            // Check if this is a valid XML entity reference
            if is_valid_entity_at(&chars, i) {
                result.push(c);
            } else {
                result.push_str("&amp;");
            }
            i += 1;
        } else if c == '<' {
            // Check if followed by digit or whitespace (not a real tag)
            if i + 1 < len {
                let next = chars[i + 1];
                if next.is_ascii_digit() || next.is_whitespace() {
                    result.push_str("&lt;");
                    i += 1;
                    continue;
                }
            }
            result.push(c);
            i += 1;
        } else {
            result.push(c);
            i += 1;
        }
    }

    result
}

/// Check if position `i` in `chars` starts a valid XML entity reference.
/// Valid forms: &amp; &lt; &gt; &quot; &apos; &#digits; &#xhex;
fn is_valid_entity_at(chars: &[char], i: usize) -> bool {
    let len = chars.len();
    if i >= len || chars[i] != '&' {
        return false;
    }

    // Look for the semicolon that ends the entity
    let mut j = i + 1;
    let mut entity_chars = Vec::new();

    while j < len && j - i < 10 {
        // Max reasonable entity length
        let c = chars[j];
        if c == ';' {
            break;
        }
        if c.is_whitespace() || c == '<' || c == '>' || c == '&' {
            // Invalid entity - hit a delimiter before semicolon
            return false;
        }
        entity_chars.push(c);
        j += 1;
    }

    // Must end with semicolon
    if j >= len || chars[j] != ';' {
        return false;
    }

    let entity: String = entity_chars.iter().collect();

    // Check known named entities
    if matches!(entity.as_str(), "amp" | "lt" | "gt" | "quot" | "apos") {
        return true;
    }

    // Check numeric character references: &#digits;
    if let Some(rest) = entity.strip_prefix('#') {
        if rest.chars().all(|c| c.is_ascii_digit()) && !rest.is_empty() {
            return true;
        }
        // Check hex: &#xhex;
        if let Some(hex_part) = rest.strip_prefix('x').or_else(|| rest.strip_prefix('X')) {
            if hex_part.chars().all(|c| c.is_ascii_hexdigit()) && !hex_part.is_empty() {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_stray_ampersand() {
        assert_eq!(sanitize_spec_xml("R&D"), "R&amp;D");
        assert_eq!(sanitize_spec_xml("A & B"), "A &amp; B");
        assert_eq!(
            sanitize_spec_xml("Tom & Jerry & Co"),
            "Tom &amp; Jerry &amp; Co"
        );
    }

    #[test]
    fn test_preserve_valid_entities() {
        assert_eq!(sanitize_spec_xml("&amp;"), "&amp;");
        assert_eq!(sanitize_spec_xml("&lt;"), "&lt;");
        assert_eq!(sanitize_spec_xml("&gt;"), "&gt;");
        assert_eq!(sanitize_spec_xml("&quot;"), "&quot;");
        assert_eq!(sanitize_spec_xml("&apos;"), "&apos;");
        assert_eq!(sanitize_spec_xml("&#65;"), "&#65;");
        assert_eq!(sanitize_spec_xml("&#x41;"), "&#x41;");
    }

    #[test]
    fn test_escape_less_than_before_digit() {
        assert_eq!(sanitize_spec_xml("<100ms"), "&lt;100ms");
        assert_eq!(sanitize_spec_xml("latency <50ms"), "latency &lt;50ms");
        assert_eq!(sanitize_spec_xml("<1 second"), "&lt;1 second");
    }

    #[test]
    fn test_escape_less_than_before_whitespace() {
        assert_eq!(sanitize_spec_xml("< 100"), "&lt; 100");
        assert_eq!(sanitize_spec_xml("value < limit"), "value &lt; limit");
    }

    #[test]
    fn test_preserve_real_tags() {
        let xml = "<project_specification><name>Test</name></project_specification>";
        assert_eq!(sanitize_spec_xml(xml), xml);
    }

    #[test]
    fn test_combined_issues() {
        let raw = "<overview>Our R&D team achieved <100ms response time for P&L reports</overview>";
        let expected =
            "<overview>Our R&amp;D team achieved &lt;100ms response time for P&amp;L reports</overview>";
        assert_eq!(sanitize_spec_xml(raw), expected);
    }

    #[test]
    fn test_already_clean_xml() {
        let clean = r#"<project_specification>
            <project_name>Clean Project</project_name>
            <overview>A well-formed specification</overview>
        </project_specification>"#;
        assert_eq!(sanitize_spec_xml(clean), clean);
    }

    #[test]
    fn test_mixed_valid_and_invalid_ampersands() {
        let input = "R&D uses &amp; and &lt; properly but Q&A doesn't";
        let expected = "R&amp;D uses &amp; and &lt; properly but Q&amp;A doesn't";
        assert_eq!(sanitize_spec_xml(input), expected);
    }

    #[test]
    fn test_success_criteria_common_pattern() {
        let input = "<success_criteria>API response time <100ms</success_criteria>";
        let expected = "<success_criteria>API response time &lt;100ms</success_criteria>";
        assert_eq!(sanitize_spec_xml(input), expected);
    }

    #[test]
    fn test_ampersand_at_end() {
        assert_eq!(sanitize_spec_xml("Test &"), "Test &amp;");
        assert_eq!(sanitize_spec_xml("&"), "&amp;");
    }

    #[test]
    fn test_incomplete_entity() {
        // &foo without semicolon should be escaped
        assert_eq!(sanitize_spec_xml("&foo bar"), "&amp;foo bar");
        // &# without digits
        assert_eq!(sanitize_spec_xml("&#abc;"), "&amp;#abc;");
    }
}
