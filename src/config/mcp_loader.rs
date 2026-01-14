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
#[allow(dead_code)]
struct McpEntry {
    #[serde(default)]
    enabled: bool,
    #[serde(default)]
    _name: Option<String>,
}

/// @returns The path to the global `opencode.jsonc` file.
/// @throws An error when the `HOME` environment variable is missing.
fn get_global_config_path() -> Result<PathBuf> {
    let home = std::env::var("HOME").context("Could not find HOME environment variable")?;
    Ok(PathBuf::from(home)
        .join(".config")
        .join("opencode")
        .join("opencode.jsonc"))
}

/// @param input The JSONC string to sanitize.
/// @returns The JSON string with comments removed.
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

/// @param mcp_entries The MCP entries from global configuration.
/// @returns The sorted list of MCP tool names.
fn collect_mcp_tool_names(mcp_entries: HashMap<String, McpEntry>) -> Vec<String> {
    let mut tools: Vec<String> = mcp_entries.into_keys().collect();
    tools.sort();
    tools
}

/// @returns A sorted list of MCP server names from global configuration.
/// @throws An error when the global config cannot be read or parsed.
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

    Ok(collect_mcp_tool_names(config.mcp))
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

    #[test]
    fn test_collect_mcp_tool_names_includes_disabled() {
        let mut entries = HashMap::new();
        entries.insert(
            "alpha".to_string(),
            McpEntry {
                enabled: false,
                _name: None,
            },
        );
        entries.insert(
            "beta".to_string(),
            McpEntry {
                enabled: true,
                _name: Some("Beta".to_string()),
            },
        );

        let tools = collect_mcp_tool_names(entries);

        assert_eq!(tools, vec!["alpha".to_string(), "beta".to_string()]);
    }
}
