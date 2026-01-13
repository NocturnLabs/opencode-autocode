use serde::Deserialize;

// ─────────────────────────────────────────────────────────────────────────────
// Security Configuration
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct SecurityConfig {
    /// Path to security allowlist file
    pub allowlist_file: String,
    /// Enforce allowlist strictly
    pub enforce_allowlist: bool,
    /// Blocked command patterns
    pub blocked_patterns: Vec<String>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            allowlist_file: ".forger/security-allowlist.json".to_string(),
            enforce_allowlist: true,
            blocked_patterns: vec![
                "rm -rf /".to_string(),
                "rm -rf ~".to_string(),
                "sudo".to_string(),
                "curl | bash".to_string(),
                "wget | bash".to_string(),
                "chmod 777".to_string(),
                "> /dev/sda".to_string(),
            ],
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// UI Configuration
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct UiConfig {
    /// Show colored output
    pub colored_output: bool,
    /// Verbose output
    pub verbose: bool,
    /// Show progress indicators
    pub show_progress: bool,
    /// Lines to show in spec preview
    pub spec_preview_lines: u32,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            colored_output: true,
            verbose: true,
            show_progress: true,
            spec_preview_lines: 25,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Notifications Configuration
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct NotificationsConfig {
    /// Webhook URL for notifications
    pub webhook_url: Option<String>,
    /// Enable webhook notifications
    pub webhook_enabled: bool,
    /// Discord Bot Token (supports interactive buttons)
    /// Loaded from $DISCORD_BOT_TOKEN env var, fallback to config value
    pub bot_token: Option<String>,
    /// Discord Channel ID for bot messages (per-project)
    pub channel_id: Option<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// MCP Tool Configuration
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct McpConfig {
    /// MCP tools in priority order
    pub priority_order: Vec<String>,
    /// Required MCP tools (set by spec generator based on project type)
    pub required_tools: Vec<String>,
    /// Prefer osgrep over grep
    pub prefer_osgrep: bool,
    /// Use sequential thinking for complex decisions
    pub use_sequential_thinking: bool,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            // Empty by default - users configure their available MCPs
            // See forger.toml for example tools with repo links
            priority_order: vec![],
            // Empty by default - spec generator populates for web projects
            required_tools: vec![],
            prefer_osgrep: true,
            use_sequential_thinking: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_defaults() {
        let security = SecurityConfig::default();
        assert!(security.enforce_allowlist);
        assert!(security.blocked_patterns.contains(&"sudo".to_string()));
    }
}
