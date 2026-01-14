use serde::Deserialize;

// ─────────────────────────────────────────────────────────────────────────────
// Security Configuration
// ─────────────────────────────────────────────────────────────────────────────

/// Security configuration for command execution and system access
///
/// Controls what commands and operations are allowed during autonomous sessions.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct SecurityConfig {
    /// Path to security allowlist file
    ///
    /// File containing list of allowed commands and operations.
    pub allowlist_file: String,

    /// Enforce allowlist strictly
    ///
    /// Whether to block any commands not explicitly in the allowlist.
    pub enforce_allowlist: bool,

    /// Blocked command patterns
    ///
    /// List of command patterns that are always blocked for security reasons.
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

/// User interface configuration
///
/// Controls the appearance and behavior of the command-line interface.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct UiConfig {
    /// Show colored output
    ///
    /// Whether to use ANSI color codes in terminal output.
    pub colored_output: bool,

    /// Verbose output
    ///
    /// Whether to show detailed logging and debug information.
    pub verbose: bool,

    /// Show progress indicators
    ///
    /// Whether to display progress bars and spinners during operations.
    pub show_progress: bool,

    /// Lines to show in spec preview
    ///
    /// Number of lines to display when previewing generated specifications.
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

/// Notifications configuration for external integrations
///
/// Controls how and where notifications about project progress are sent.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct NotificationsConfig {
    /// Webhook URL for notifications
    ///
    /// URL where notifications will be sent via webhook.
    pub webhook_url: Option<String>,

    /// Enable webhook notifications
    ///
    /// Whether to send notifications via webhook.
    pub webhook_enabled: bool,

    /// Discord Bot Token (supports interactive buttons)
    /// Loaded from $DISCORD_BOT_TOKEN env var, fallback to config value
    ///
    /// Authentication token for Discord bot integration.
    pub bot_token: Option<String>,

    /// Discord Channel ID for bot messages (per-project)
    ///
    /// Specific Discord channel where bot messages should be sent.
    pub channel_id: Option<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// MCP Tool Configuration
// ─────────────────────────────────────────────────────────────────────────────

/// MCP (Multi-Cognitive Processing) tool configuration
///
/// Controls which cognitive tools are available and how they're prioritized.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct McpConfig {
    /// MCP tools in priority order
    ///
    /// List of available MCP tools, ordered by priority.
    pub priority_order: Vec<String>,

    /// Required MCP tools (set by spec generator based on project type)
    ///
    /// Tools that are required for the current project type.
    pub required_tools: Vec<String>,

    /// Use sequential thinking for complex decisions
    ///
    /// Whether to use sequential reasoning for complex decision-making.
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
