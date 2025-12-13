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
    pub paths: PathsConfig,
    pub autonomous: AutonomousConfig,
    pub agent: AgentConfig,
    pub verbalized_sampling: VerbalizedSamplingConfig,
    pub mcp: McpConfig,
    pub features: FeaturesConfig,
    pub scaffolding: ScaffoldingConfig,
    pub security: SecurityConfig,
    pub ui: UiConfig,
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
                "/usr/local/bin/opencode".to_string(),
            ],
            log_dir: "$HOME/Work/local-work/opencode-logs".to_string(),
            vs_cache_dir: ".vs-cache".to_string(),
            progress_file: "opencode-progress.txt".to_string(),
            feature_list_file: "feature_list.json".to_string(),
            app_spec_file: "app_spec.md".to_string(),
        }
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
// Verbalized Sampling Configuration
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct VerbalizedSamplingConfig {
    /// Enable verbalized sampling
    pub enabled: bool,
    /// Number of approaches to generate
    pub num_approaches: u32,
    /// Probability range for conventional approaches [min, max]
    pub conventional_probability: [f32; 2],
    /// Probability range for alternative approaches [min, max]
    pub alternative_probability: [f32; 2],
    /// Probability range for creative approaches [min, max]
    pub creative_probability: [f32; 2],
    /// Minimum conventional approaches
    pub min_conventional: u32,
    /// Minimum alternative approaches
    pub min_alternative: u32,
    /// Minimum creative approaches
    pub min_creative: u32,
    /// Cache VS results
    pub cache_results: bool,
}

impl Default for VerbalizedSamplingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            num_approaches: 10,
            conventional_probability: [0.8, 0.9],
            alternative_probability: [0.4, 0.6],
            creative_probability: [0.0, 0.2],
            min_conventional: 2,
            min_alternative: 3,
            min_creative: 3,
            cache_results: true,
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
    /// Prefer mgrep over grep
    pub prefer_mgrep: bool,
    /// Use sequential thinking for complex decisions
    pub use_sequential_thinking: bool,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            priority_order: vec![
                "mgrep".to_string(),
                "chat-history".to_string(),
                "deepwiki".to_string(),
                "perplexica".to_string(),
                "sequential-thinking".to_string(),
            ],
            prefer_mgrep: true,
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
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Environment Variable Expansion
// ─────────────────────────────────────────────────────────────────────────────

/// Expand environment variables in a string (e.g., $HOME, ${HOME})
fn expand_env_var(s: &str) -> String {
    let mut result = s.to_string();

    // Handle $HOME specifically (most common case)
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
                while let Some(&ch) = chars.peek() {
                    if ch == '}' {
                        chars.next();
                        break;
                    }
                    var_name.push(chars.next().unwrap());
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
        assert!(config.verbalized_sampling.enabled);
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

[verbalized_sampling]
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
        assert!(!config.verbalized_sampling.enabled);
        assert_eq!(config.verbalized_sampling.num_approaches, 5);
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
    fn test_verbalized_sampling_defaults() {
        let vs = VerbalizedSamplingConfig::default();
        assert_eq!(vs.num_approaches, 10);
        assert_eq!(vs.conventional_probability, [0.8, 0.9]);
        assert_eq!(vs.min_conventional, 2);
    }

    #[test]
    fn test_security_defaults() {
        let security = SecurityConfig::default();
        assert!(security.enforce_allowlist);
        assert!(security.blocked_patterns.contains(&"sudo".to_string()));
    }
}
