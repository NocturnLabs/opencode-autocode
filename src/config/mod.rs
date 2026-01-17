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
pub use environment::{McpConfig, NotificationsConfig, SecurityConfig, UiConfig};
pub use project::{
    ComplexityLevel, FeaturesConfig, GenerationConfig, GenerationRequirements, ModelsConfig,
    PathsConfig, ScaffoldingConfig,
};

/// Default config filename
const CONFIG_FILENAME: &str = "forger.toml";

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
}

// ─────────────────────────────────────────────────────────────────────────────
// Config Loading Implementation
// ─────────────────────────────────────────────────────────────────────────────

impl Config {
    /// Resolve the config path.
    ///
    /// # Arguments
    ///
    /// * `dir` - Optional directory path to search for config files
    ///
    /// # Returns
    ///
    /// PathBuf containing the resolved configuration file path
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opencode_forger::config::Config;
    /// use std::path::Path;
    /// let config_path = Config::resolve_config_path(Some(Path::new("/my/project")));
    /// ```
    pub fn resolve_config_path(dir: Option<&Path>) -> PathBuf {
        resolve_config_path(dir)
    }

    /// Load configuration from the specified directory or search upwards for project root.
    ///
    /// Reads from `forger.toml` if it exists; otherwise returns default configuration.
    /// Logs a deprecation warning if `.forger/config.toml` exists.
    ///
    /// # Arguments
    ///
    /// * `dir` - Optional directory path to search for config files
    ///
    /// # Returns
    ///
    /// Result containing the loaded Config instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opencode_forger::config::Config;
    /// use std::path::Path;
    /// // Load config from current directory
    /// let config = Config::load(None);
    ///
    /// // Load config from specific directory
    /// let config = Config::load(Some(Path::new("/my/project")));
    /// ```
    pub fn load(dir: Option<&Path>) -> Result<Self> {
        let root = match dir {
            Some(d) => Some(d.to_path_buf()),
            None => find_project_root().or_else(|| env::current_dir().ok()),
        };

        let config_path = resolve_config_path(root.as_deref());

        // Check for legacy config and log deprecation warning
        if let Some(ref r) = root {
            let legacy_config = r.join(".forger").join("config.toml");
            if legacy_config.exists() {
                eprintln!("⚠️  WARNING: .forger/config.toml is deprecated and will be ignored.");
                eprintln!(
                    "   Please migrate your settings to {} at the project root.",
                    CONFIG_FILENAME
                );
                eprintln!(
                    "   See https://github.com/NocturnLabs/opencode-forger for migration guide."
                );
            }
        }

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
            config.apply_runtime_settings();

            Ok(config)
        } else {
            let config = Config::default();
            config.apply_runtime_settings();
            Ok(config)
        }
    }

    /// Load configuration from a specific file path.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the configuration file
    ///
    /// # Returns
    ///
    /// Result containing the loaded Config instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opencode_forger::config::Config;
    /// use std::path::Path;
    /// let config = Config::load_from_file(Path::new("/path/to/forger.toml"));
    /// ```
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let mut config: Config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

        config.expand_env_vars();
        config.apply_runtime_settings();
        Ok(config)
    }

    /// Expand environment variables in path-like config values.
    ///
    /// Replaces environment variables like $HOME, ${HOME}, %APPDATA%, etc. with their actual values.
    /// Also handles Discord bot token loading from environment variable.
    fn expand_env_vars(&mut self) {
        self.paths.log_dir = expand_env_var(&self.paths.log_dir);
        self.paths.vs_cache_dir = expand_env_var(&self.paths.vs_cache_dir);
        self.paths.database_file = expand_env_var(&self.paths.database_file);
        self.paths.app_spec_file = expand_env_var(&self.paths.app_spec_file);
        self.paths.opencode_paths = self
            .paths
            .opencode_paths
            .iter()
            .map(|p| expand_env_var(p))
            .collect();
        self.scaffolding.output_dir = expand_env_var(&self.scaffolding.output_dir);
        self.security.allowlist_file = expand_env_var(&self.security.allowlist_file);
        self.alternative_approaches.cache_dir =
            expand_env_var(&self.alternative_approaches.cache_dir);
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
    ///
    /// Converts relative paths to absolute paths by joining them with the project root directory.
    ///
    /// # Arguments
    ///
    /// * `root` - The project root directory path
    fn canonicalize_paths(&mut self, root: &Path) {
        let canonicalize = |p: &str| -> String {
            if p.trim().is_empty() {
                return p.to_string();
            }
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
        self.paths.log_dir = canonicalize(&self.paths.log_dir);
        self.scaffolding.output_dir = canonicalize(&self.scaffolding.output_dir);
        self.alternative_approaches.cache_dir =
            canonicalize(&self.alternative_approaches.cache_dir);
        self.conductor.context_dir = canonicalize(&self.conductor.context_dir);
        self.conductor.tracks_dir = canonicalize(&self.conductor.tracks_dir);
    }

    /// Apply configuration-driven runtime settings.
    ///
    /// Sets global application settings based on configuration values.
    fn apply_runtime_settings(&self) {
        crate::theming::set_colored_output(self.ui.colored_output);
    }
}

/// Resolve the config file path.
///
/// Searches for `forger.toml` in the specified root directory,
/// or returns the preferred path if it doesn't exist.
///
/// # Arguments
///
/// * `root` - Optional root directory to search for config files
///
/// # Returns
///
/// PathBuf containing the resolved configuration file path
fn resolve_config_path(root: Option<&Path>) -> PathBuf {
    let resolve_from_root = |root: &Path| root.join(CONFIG_FILENAME);

    match root {
        Some(root) => resolve_from_root(root),
        None => PathBuf::from(CONFIG_FILENAME),
    }
}

/// Search upwards for the project root (containing .forger directory).
///
/// Walks up the directory tree from the current working directory until it finds
/// a directory containing a `.forger` subdirectory, which indicates the project root.
///
/// # Returns
///
/// Option containing the project root path if found, None otherwise
///
/// # Examples
///
/// ```rust
/// use opencode_forger::config::find_project_root;
/// if let Some(root) = find_project_root() {
///     println!("Project root: {}", root.display());
/// }
/// ```
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
///
/// Supports multiple environment variable formats:
/// - Unix-style: $VAR, ${VAR}
/// - Windows-style: %VAR%
///
/// # Arguments
///
/// * `s` - Input string containing environment variable references
///
/// # Returns
///
/// String with environment variables replaced by their values
///
/// # Examples
///
/// ```rust,ignore
/// let expanded = expand_env_var("$HOME/.config");
/// // Returns something like "/home/user/.config"
/// ```
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
///
/// Parses ${VAR} style environment variables and replaces them with their values.
/// If a variable is not found, leaves the original reference intact.
///
/// # Arguments
///
/// * `s` - Input string containing ${VAR} style environment variables
///
/// # Returns
///
/// String with ${VAR} style variables replaced by their values
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

#[test]
fn test_config_loader_with_no_config_file() {
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let _forger_toml = temp_dir.path().join("forger.toml");
    let legacy_dir = temp_dir.path().join(".forger");
    std::fs::create_dir_all(&legacy_dir).unwrap();

    // Write no config files
    let config = Config::load(Some(temp_dir.path())).unwrap();

    // Check that default was used
    assert_eq!(config.models.default, "opencode/glm-4.7-free");
    assert_eq!(config.models.autonomous, "opencode/minimax-m2.1-free");
}

#[test]
fn test_config_loader_ignores_legacy_config() {
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let _forger_toml = temp_dir.path().join("forger.toml");
    let legacy_dir = temp_dir.path().join(".forger");
    std::fs::create_dir_all(&legacy_dir).unwrap();
    let legacy_config = legacy_dir.join("config.toml");

    // Write both config files
    std::fs::write(
        &_forger_toml,
        r#"[models]
default = "test-model"
"#,
    )
    .unwrap();
    std::fs::write(
        &legacy_config,
        r#"[models]
default = "legacy-model"
"#,
    )
    .unwrap();

    // Load config and check that forger.toml was used, not legacy
    let config = Config::load(Some(temp_dir.path())).unwrap();

    // Check that forger.toml was used
    assert_eq!(config.models.default, "test-model");
    assert!(!config.models.default.contains("legacy-model"));
}
