#![allow(dead_code)]
//! Configuration loading and management
//!
//! Loads configuration from `autocode.toml` at the project root,
//! with fallback to sensible defaults.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Default config filename
const CONFIG_FILENAME: &str = "autocode.toml";

// ─────────────────────────────────────────────────────────────────────────────
// Main Config Struct
// ─────────────────────────────────────────────────────────────────────────────

/// Application configuration - all sections from autocode.toml
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    pub models: ModelsConfig,
    pub generation: GenerationConfig,
    pub paths: PathsConfig,
    pub autonomous: AutonomousConfig,
    pub agent: AgentConfig,
    pub alternative_approaches: AlternativeApproachesConfig,
    pub mcp: McpConfig,
    pub features: FeaturesConfig,
    pub scaffolding: ScaffoldingConfig,
    pub security: SecurityConfig,
    pub ui: UiConfig,
    pub notifications: NotificationsConfig,
}

// ─────────────────────────────────────────────────────────────────────────────
// Models Configuration
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ModelsConfig {
    /// Default model for spec generation
    pub default: String,
    /// Model for autonomous coding sessions
    pub autonomous: String,
    /// Model for reasoning/planning tasks
    pub reasoning: String,
    /// Model for enhancement discovery
    pub enhancement: String,
}

impl Default for ModelsConfig {
    fn default() -> Self {
        Self {
            default: "opencode/big-pickle".to_string(),
            autonomous: "opencode/grok-code".to_string(),
            reasoning: "opencode/grok-code".to_string(),
            enhancement: "opencode/big-pickle".to_string(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Generation Configuration
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct GenerationConfig {
    /// Complexity level: "comprehensive" or "minimal"
    pub complexity: String,
    /// Minimum features for comprehensive mode
    pub min_features: u32,
    /// Minimum database tables
    pub min_database_tables: u32,
    /// Minimum API endpoints
    pub min_api_endpoints: u32,
    /// Minimum implementation steps
    pub min_implementation_steps: u32,
    /// Minimum features for minimal mode
    pub minimal_min_features: u32,
    /// Minimum database tables for minimal mode
    pub minimal_min_database_tables: u32,
    /// Minimum API endpoints for minimal mode
    pub minimal_min_api_endpoints: u32,
    /// Minimum implementation steps for minimal mode
    pub minimal_min_implementation_steps: u32,
    /// Include security section
    pub include_security_section: bool,
    /// Include testing strategy section
    pub include_testing_strategy: bool,
    /// Include devops section
    pub include_devops_section: bool,
    /// Include accessibility section
    pub include_accessibility: bool,
    /// Include future enhancements section
    pub include_future_enhancements: bool,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            complexity: "comprehensive".to_string(),
            min_features: 15,
            min_database_tables: 10,
            min_api_endpoints: 30,
            min_implementation_steps: 8,
            minimal_min_features: 5,
            minimal_min_database_tables: 3,
            minimal_min_api_endpoints: 10,
            minimal_min_implementation_steps: 4,
            include_security_section: true,
            include_testing_strategy: true,
            include_devops_section: true,
            include_accessibility: true,
            include_future_enhancements: true,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Paths Configuration
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct PathsConfig {
    /// Paths to search for opencode executable
    pub opencode_paths: Vec<String>,
    /// Log directory for autonomous sessions
    pub log_dir: String,
    /// Cache directory for verbalized sampling results
    pub vs_cache_dir: String,
    /// Progress file name
    pub progress_file: String,
    /// Feature list file name
    pub feature_list_file: String,
    /// App spec file name
    pub app_spec_file: String,
}

impl Default for PathsConfig {
    fn default() -> Self {
        Self {
            opencode_paths: vec![
                "opencode".to_string(),
                #[cfg(not(target_os = "windows"))]
                "/usr/local/bin/opencode".to_string(),
            ],
            log_dir: get_default_log_dir(),
            vs_cache_dir: ".vs-cache".to_string(),
            progress_file: "opencode-progress.txt".to_string(),
            feature_list_file: "feature_list.json".to_string(),
            app_spec_file: "app_spec.md".to_string(),
        }
    }
}

/// Get the platform-appropriate OpenCode log directory
fn get_default_log_dir() -> String {
    #[cfg(target_os = "windows")]
    {
        "%APPDATA%\\opencode\\log".to_string()
    }
    #[cfg(not(target_os = "windows"))]
    {
        "$HOME/.local/share/opencode/log".to_string()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Autonomous Session Configuration
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AutonomousConfig {
    /// Seconds between autonomous sessions
    pub delay_between_sessions: u32,
    /// Maximum iterations (0 = unlimited)
    pub max_iterations: u32,
    /// Log level for opencode commands
    pub log_level: String,
    /// Session timeout in minutes (0 = no timeout)
    pub session_timeout_minutes: u32,
    /// Auto-commit after feature completion
    pub auto_commit: bool,
}

impl Default for AutonomousConfig {
    fn default() -> Self {
        Self {
            delay_between_sessions: 5,
            max_iterations: 0,
            log_level: "DEBUG".to_string(),
            session_timeout_minutes: 60,
            auto_commit: true,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Agent Behavior Configuration
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AgentConfig {
    /// Max retry attempts before research protocol
    pub max_retry_attempts: u32,
    /// Max research-based attempts before moving on
    pub max_research_attempts: u32,
    /// Number of passing features to verify before new work
    pub verification_sample_size: u32,
    /// Focus on one feature at a time
    pub single_feature_focus: bool,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_retry_attempts: 3,
            max_research_attempts: 3,
            verification_sample_size: 2,
            single_feature_focus: true,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Alternative Approaches Configuration (Stuck Recovery)
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AlternativeApproachesConfig {
    /// Enable alternative approach generation when stuck
    pub enabled: bool,
    /// Number of alternative approaches to generate
    pub num_approaches: u32,
    /// Retry threshold before triggering alternative generation
    pub retry_threshold: u32,
    /// Cache results to avoid regenerating
    pub cache_results: bool,
    /// Cache directory
    pub cache_dir: String,
}

impl Default for AlternativeApproachesConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            num_approaches: 7,
            retry_threshold: 3,
            cache_results: true,
            cache_dir: ".approach-cache".to_string(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// MCP Tool Configuration
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct McpConfig {
    /// MCP tools in priority order
    pub priority_order: Vec<String>,
    /// Prefer osgrep over grep
    pub prefer_osgrep: bool,
    /// Use sequential thinking for complex decisions
    pub use_sequential_thinking: bool,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            // Empty by default - users configure their available MCPs
            // See autocode.toml for example tools with repo links
            priority_order: vec![],
            prefer_osgrep: false,
            use_sequential_thinking: true,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Features Configuration
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct FeaturesConfig {
    /// Valid feature categories
    pub categories: Vec<String>,
    /// Priority levels
    pub priorities: Vec<String>,
    /// Require verification_command
    pub require_verification_command: bool,
    /// Min steps for narrow tests
    pub narrow_test_min_steps: u32,
    /// Max steps for narrow tests
    pub narrow_test_max_steps: u32,
    /// Min steps for comprehensive tests
    pub comprehensive_test_min_steps: u32,
}

impl Default for FeaturesConfig {
    fn default() -> Self {
        Self {
            categories: vec![
                "functional".to_string(),
                "style".to_string(),
                "integration".to_string(),
                "performance".to_string(),
                "enhancement".to_string(),
            ],
            priorities: vec![
                "critical".to_string(),
                "high".to_string(),
                "medium".to_string(),
                "low".to_string(),
            ],
            require_verification_command: false,
            narrow_test_min_steps: 2,
            narrow_test_max_steps: 5,
            comprehensive_test_min_steps: 10,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Scaffolding Configuration
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ScaffoldingConfig {
    /// Initialize git repository
    pub git_init: bool,
    /// Default output directory
    pub output_dir: String,
    /// Create .opencode directory
    pub create_opencode_dir: bool,
    /// Create scripts directory
    pub create_scripts_dir: bool,
}

impl Default for ScaffoldingConfig {
    fn default() -> Self {
        Self {
            git_init: false,
            output_dir: String::new(),
            create_opencode_dir: true,
            create_scripts_dir: true,
        }
    }
}

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
            allowlist_file: "scripts/security-allowlist.json".to_string(),
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
            verbose: false,
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
}

// ─────────────────────────────────────────────────────────────────────────────
// Default Implementation for Main Config
// ─────────────────────────────────────────────────────────────────────────────

// ─────────────────────────────────────────────────────────────────────────────
// Config Loading Implementation
// ─────────────────────────────────────────────────────────────────────────────

impl Config {
    /// Load configuration from the specified directory or current directory.
    pub fn load(dir: Option<&Path>) -> Result<Self> {
        let config_path = match dir {
            Some(d) => d.join(CONFIG_FILENAME),
            None => PathBuf::from(CONFIG_FILENAME),
        };

        if config_path.exists() {
            let content = fs::read_to_string(&config_path).with_context(|| {
                format!("Failed to read config file: {}", config_path.display())
            })?;

            let mut config: Config = toml::from_str(&content).with_context(|| {
                format!("Failed to parse config file: {}", config_path.display())
            })?;

            config.expand_env_vars();
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }

    /// Load configuration from a specific file path.
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let mut config: Config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

        config.expand_env_vars();
        Ok(config)
    }

    /// Expand environment variables in path-like config values.
    fn expand_env_vars(&mut self) {
        self.paths.log_dir = expand_env_var(&self.paths.log_dir);
        self.paths.vs_cache_dir = expand_env_var(&self.paths.vs_cache_dir);
        self.paths.opencode_paths = self
            .paths
            .opencode_paths
            .iter()
            .map(|p| expand_env_var(p))
            .collect();
        self.scaffolding.output_dir = expand_env_var(&self.scaffolding.output_dir);
        self.security.allowlist_file = expand_env_var(&self.security.allowlist_file);
        if let Some(ref url) = self.notifications.webhook_url {
            self.notifications.webhook_url = Some(expand_env_var(url));
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Environment Variable Expansion
// ─────────────────────────────────────────────────────────────────────────────

/// Expand environment variables in a string (e.g., $HOME, ${HOME}, %APPDATA%)
fn expand_env_var(s: &str) -> String {
    let mut result = s.to_string();

    // Handle Windows %APPDATA%
    if let Ok(appdata) = env::var("APPDATA") {
        result = result.replace("%APPDATA%", &appdata);
    }

    // Handle Windows %USERPROFILE%
    if let Ok(userprofile) = env::var("USERPROFILE") {
        result = result.replace("%USERPROFILE%", &userprofile);
    }

    // Handle $HOME specifically (most common case on Unix)
    if let Ok(home) = env::var("HOME") {
        result = result.replace("$HOME", &home);
        result = result.replace("${HOME}", &home);
    }

    regex_lite_expand(&result)
}

/// Simple environment variable expansion without regex dependency
fn regex_lite_expand(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '$' {
            if chars.peek() == Some(&'{') {
                chars.next();
                let mut var_name = String::new();
                for ch in chars.by_ref() {
                    if ch == '}' {
                        break;
                    }
                    var_name.push(ch);
                }
                if let Ok(val) = env::var(&var_name) {
                    result.push_str(&val);
                } else {
                    result.push_str(&format!("${{{}}}", var_name));
                }
            } else {
                result.push(c);
            }
        } else {
            result.push(c);
        }
    }

    result
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.models.default, "opencode/big-pickle");
        assert_eq!(config.models.autonomous, "opencode/grok-code");
        assert_eq!(config.autonomous.delay_between_sessions, 5);
        assert_eq!(config.agent.max_retry_attempts, 3);
        assert!(config.alternative_approaches.enabled);
        assert_eq!(config.ui.spec_preview_lines, 25);
    }

    #[test]
    fn test_load_missing_file_returns_default() {
        let config = Config::load(Some(Path::new("/nonexistent/path"))).unwrap();
        assert_eq!(config.models.default, "opencode/big-pickle");
    }

    #[test]
    fn test_load_custom_config() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
[models]
default = "custom/model"
autonomous = "custom/auto"

[autonomous]
delay_between_sessions = 10

[agent]
max_retry_attempts = 5

[alternative_approaches]
enabled = false
num_approaches = 5
"#
        )
        .unwrap();

        let config = Config::load_from_file(file.path()).unwrap();
        assert_eq!(config.models.default, "custom/model");
        assert_eq!(config.models.autonomous, "custom/auto");
        assert_eq!(config.autonomous.delay_between_sessions, 10);
        assert_eq!(config.agent.max_retry_attempts, 5);
        assert!(!config.alternative_approaches.enabled);
        assert_eq!(config.alternative_approaches.num_approaches, 5);
        // Check defaults preserved
        assert_eq!(config.autonomous.max_iterations, 0);
        assert_eq!(config.ui.spec_preview_lines, 25);
    }

    #[test]
    fn test_expand_env_var() {
        std::env::set_var("TEST_VAR", "test_value");
        let result = expand_env_var("${TEST_VAR}/path");
        assert_eq!(result, "test_value/path");
    }

    #[test]
    fn test_alternative_approaches_defaults() {
        let aa = AlternativeApproachesConfig::default();
        assert_eq!(aa.num_approaches, 7);
        assert_eq!(aa.retry_threshold, 3);
        assert!(aa.cache_results);
    }

    #[test]
    fn test_security_defaults() {
        let security = SecurityConfig::default();
        assert!(security.enforce_allowlist);
        assert!(security.blocked_patterns.contains(&"sudo".to_string()));
    }
}
