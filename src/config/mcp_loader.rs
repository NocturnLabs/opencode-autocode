use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

/// Structure for the global OpenCode config
#[derive(Debug, Deserialize)]
struct GlobalConfig {
    #[serde(default)]
    mcp: HashMap<String, McpEntry>,
}

#[derive(Debug, Deserialize)]
struct McpEntry {
    #[serde(default)]
    enabled: bool,
    #[serde(default)]
    _name: Option<String>,
}

/// Helper to get the global config path
fn get_global_config_path() -> Result<PathBuf> {
    let home = std::env::var("HOME").context("Could not find HOME environment variable")?;
    Ok(PathBuf::from(home)
        .join(".config")
        .join("opencode")
        .join("opencode.jsonc"))
}

/// Strip JSONC comments (// and /* */)
fn strip_jsonc_comments(input: &str) -> String {
    let mut output = String::new();
    let mut chars = input.chars().peekable();
    let mut in_string = false;
    let mut in_multiline_comment = false;

    while let Some(c) = chars.next() {
        if in_multiline_comment {
            if c == '*' && chars.peek() == Some(&'/') {
                chars.next(); // consume '/'
                in_multiline_comment = false;
            }
            continue;
        }

        if in_string {
            output.push(c);
            if c == '\\' {
                if let Some(next) = chars.next() {
                    output.push(next);
                }
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }

        // Not in string or comment
        if c == '"' {
            in_string = true;
            output.push(c);
        } else if c == '/' {
            if let Some(next) = chars.peek() {
                if *next == '/' {
                    // Single line comment: skip until newline
                    chars.next();
                    while let Some(n) = chars.peek() {
                        if *n == '\n' {
                            break;
                        }
                        chars.next();
                    }
                    continue;
                } else if *next == '*' {
                    // Multiline comment
                    chars.next();
                    in_multiline_comment = true;
                    continue;
                }
            }
            output.push(c);
        } else {
            output.push(c);
        }
    }
    output
}

/// Fetch available global MCP servers
/// Returns a list of server names that are enabled in global config
pub fn load_global_mcp_servers() -> Result<Vec<String>> {
    let path = get_global_config_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read global config: {}", path.display()))?;

    let json = strip_jsonc_comments(&content);

    let config: GlobalConfig = serde_json::from_str(&json)
        .with_context(|| format!("Failed to parse global config: {}", path.display()))?;

    let mut tools: Vec<String> = config
        .mcp
        .into_iter()
        .filter(|(_, v)| v.enabled)
        .map(|(k, _)| k)
        .collect();

    tools.sort();
    Ok(tools)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_comments() {
        let jsonc = r#"{
    "key": "value", // comment
    "url": "http://example.com",
    /* multiline
       comment */
    "nested": {
        "quoted_slashes": "foo // bar"
    }
}"#;
        let json = strip_jsonc_comments(jsonc);
        assert!(json.contains(r#""key": "value""#));
        assert!(json.contains(r#""url": "http://example.com""#));
        assert!(json.contains(r#""quoted_slashes": "foo // bar""#));
        assert!(!json.contains("comment"));
    }
}
