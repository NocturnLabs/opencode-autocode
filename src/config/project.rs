use serde::{Deserialize, Serialize};
use std::fmt;

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
    /// We use this model specifically when retrying failed spec generations.
    /// It needs to be good at adhering to strict output formats (XML/JSON) to "fix" what the creative model broke.
    pub fixer: String,
}

impl Default for ModelsConfig {
    fn default() -> Self {
        Self {
            default: "opencode/glm-4.7-free".to_string(),
            autonomous: "opencode/minimax-m2.1-free".to_string(),
            reasoning: "opencode/glm-4.7-free".to_string(),
            enhancement: "opencode/glm-4.7-free".to_string(),
            fixer: "opencode/grok-code".to_string(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Generation Configuration
// ─────────────────────────────────────────────────────────────────────────────

/// Complexity level selection for generated specs.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ComplexityLevel {
    /// Comprehensive specs cover production-ready detail.
    Comprehensive,
    /// Minimal specs focus only on the core needs.
    Minimal,
}

impl Default for ComplexityLevel {
    fn default() -> Self {
        ComplexityLevel::Comprehensive
    }
}

impl ComplexityLevel {
    /// Return the lowercase representation used in config files.
    pub fn as_str(&self) -> &'static str {
        match self {
            ComplexityLevel::Comprehensive => "comprehensive",
            ComplexityLevel::Minimal => "minimal",
        }
    }

    /// Toggle between the two supported complexity levels.
    pub fn toggle(&self) -> Self {
        match self {
            ComplexityLevel::Comprehensive => ComplexityLevel::Minimal,
            ComplexityLevel::Minimal => ComplexityLevel::Comprehensive,
        }
    }

    /// Parse a string into the corresponding complexity level, defaulting to comprehensive.
    pub fn from_str(value: &str) -> Self {
        match value.trim().to_lowercase().as_str() {
            "minimal" => ComplexityLevel::Minimal,
            _ => ComplexityLevel::Comprehensive,
        }
    }
}

impl fmt::Display for ComplexityLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct GenerationConfig {
    /// Complexity level guiding generated specs.
    pub complexity: ComplexityLevel,
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
    /// Enable parallel subagent spec generation (faster but uses more tokens)
    pub enable_subagents: bool,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            complexity: ComplexityLevel::default(),
            include_security_section: true,
            include_testing_strategy: true,
            include_devops_section: true,
            include_accessibility: true,
            include_future_enhancements: true,
            enable_subagents: true,
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
    /// SQLite database file name
    pub database_file: String,
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
            database_file: ".forger/progress.db".to_string(),
            app_spec_file: ".forger/app_spec.md".to_string(),
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
