//! Configuration loading and management
//!
//! Loads configuration from `forger.toml` at the project root,
//! with fallback to sensible defaults.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub mod autonomous;
pub mod environment;
pub mod mcp_loader;
pub mod project;

pub use autonomous::{AgentConfig, AlternativeApproachesConfig, AutonomousConfig, ConductorConfig};
pub use environment::{
    CommunicationConfig, McpConfig, NotificationsConfig, SecurityConfig, UiConfig,
};
pub use project::{FeaturesConfig, GenerationConfig, ModelsConfig, PathsConfig, ScaffoldingConfig};

/// Default config filename
const CONFIG_FILENAME: &str = ".forger/config.toml";

// ─────────────────────────────────────────────────────────────────────────────
// Main Config Struct
// ─────────────────────────────────────────────────────────────────────────────

/// Application configuration - all sections from forger.toml
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
    pub conductor: ConductorConfig,
    pub communication: CommunicationConfig,
}

// ─────────────────────────────────────────────────────────────────────────────
// Config Loading Implementation
// ─────────────────────────────────────────────────────────────────────────────

impl Config {
    /// Load configuration from the specified directory or search upwards for project root.
    pub fn load(dir: Option<&Path>) -> Result<Self> {
        let root = match dir {
            Some(d) => Some(d.to_path_buf()),
            None => find_project_root(),
        };

        let config_path = match root {
            Some(ref r) => r.join(CONFIG_FILENAME),
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

            // Canonicalize relative paths based on discovered root
            if let Some(r) = root {
                config.canonicalize_paths(&r);
            }

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
        self.conductor.context_dir = expand_env_var(&self.conductor.context_dir);
        self.conductor.tracks_dir = expand_env_var(&self.conductor.tracks_dir);
        if let Some(ref url) = self.notifications.webhook_url {
            self.notifications.webhook_url = Some(expand_env_var(url));
        }
        // Bot token: check env var first ($DISCORD_BOT_TOKEN), then config value
        if self
            .notifications
            .bot_token
            .as_ref()
            .is_none_or(|t| t.is_empty())
        {
            if let Ok(token) = std::env::var("DISCORD_BOT_TOKEN") {
                self.notifications.bot_token = Some(token);
            }
        } else if let Some(ref token) = self.notifications.bot_token {
            self.notifications.bot_token = Some(expand_env_var(token));
        }
        // Channel ID from config (per-project)
        if let Some(ref channel) = self.notifications.channel_id {
            self.notifications.channel_id = Some(expand_env_var(channel));
        }
    }

    /// Canonicalize relative paths based on the project root.
    fn canonicalize_paths(&mut self, root: &Path) {
        let canonicalize = |p: &str| -> String {
            let path = Path::new(p);
            if path.is_relative() {
                root.join(path).to_string_lossy().to_string()
            } else {
                p.to_string()
            }
        };

        self.paths.database_file = canonicalize(&self.paths.database_file);
        self.paths.app_spec_file = canonicalize(&self.paths.app_spec_file);
        self.paths.vs_cache_dir = canonicalize(&self.paths.vs_cache_dir);
        self.conductor.context_dir = canonicalize(&self.conductor.context_dir);
        self.conductor.tracks_dir = canonicalize(&self.conductor.tracks_dir);
    }
}

/// Search upwards for the project root (containing .forger directory).
pub fn find_project_root() -> Option<PathBuf> {
    let current_dir = env::current_dir().ok()?;
    let mut current = current_dir.as_path();

    loop {
        if current.join(".forger").is_dir() {
            return Some(current.to_path_buf());
        }

        match current.parent() {
            Some(parent) => current = parent,
            None => break,
        }
    }

    None
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
        assert_eq!(config.models.default, "opencode/glm-4.7-free");
        assert_eq!(config.models.autonomous, "opencode/minimax-m2.1-free");
        assert_eq!(config.autonomous.delay_between_sessions, 5);
        assert_eq!(config.autonomous.session_timeout_minutes, 15);
        assert_eq!(config.agent.max_retry_attempts, 3);
        assert!(config.alternative_approaches.enabled);
        assert_eq!(config.ui.spec_preview_lines, 25);
    }

    #[test]
    fn test_load_missing_file_returns_default() {
        let config = Config::load(Some(Path::new("/nonexistent/path"))).unwrap();
        assert_eq!(config.models.default, "opencode/glm-4.7-free");
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
}
