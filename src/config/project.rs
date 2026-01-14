use serde::{Deserialize, Serialize};
use std::fmt;

// ─────────────────────────────────────────────────────────────────────────────
// Models Configuration
// ─────────────────────────────────────────────────────────────────────────────

/// Configuration for AI model selection
///
/// Specifies which models to use for different types of tasks in the code generation process.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ModelsConfig {
    /// Default model for spec generation
    ///
    /// Used for generating project specifications from user requirements.
    pub default: String,

    /// Model for autonomous coding sessions
    ///
    /// Used when the AI is working autonomously to implement features.
    pub autonomous: String,

    /// Model for reasoning/planning tasks
    ///
    /// Used for complex reasoning and planning activities.
    pub reasoning: String,

    /// Model for enhancement discovery
    ///
    /// Used for identifying potential enhancements and improvements.
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
///
/// Determines the level of detail and comprehensiveness in generated specifications.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ComplexityLevel {
    /// Comprehensive specs cover production-ready detail.
    ///
    /// Includes extensive features, database tables, API endpoints, and implementation steps.
    #[default]
    Comprehensive,

    /// Minimal specs focus only on the core needs.
    ///
    /// Includes only essential features and basic structure.
    Minimal,
}

impl ComplexityLevel {
    /// Return the lowercase representation used in config files.
    ///
    /// # Returns
    ///
    /// String slice containing the lowercase name of the complexity level
    ///
    /// # Examples
    ///
    /// ```rust
    /// let level = ComplexityLevel::Comprehensive;
    /// assert_eq!(level.as_str(), "comprehensive");
    /// ```
    pub fn as_str(&self) -> &'static str {
        match self {
            ComplexityLevel::Comprehensive => "comprehensive",
            ComplexityLevel::Minimal => "minimal",
        }
    }

    /// Toggle between the two supported complexity levels.
    ///
    /// # Returns
    ///
    /// The opposite complexity level
    ///
    /// # Examples
    ///
    /// ```rust
    /// let comprehensive = ComplexityLevel::Comprehensive;
    /// let minimal = comprehensive.toggle();
    /// assert_eq!(minimal, ComplexityLevel::Minimal);
    /// ```
    pub fn toggle(&self) -> Self {
        match self {
            ComplexityLevel::Comprehensive => ComplexityLevel::Minimal,
            ComplexityLevel::Minimal => ComplexityLevel::Comprehensive,
        }
    }
}

impl std::str::FromStr for ComplexityLevel {
    type Err = ();

    /// Parse a string into a complexity level, defaulting to comprehensive.
    ///
    /// # Arguments
    ///
    /// * `value` - String slice containing the complexity level name
    ///
    /// # Returns
    ///
    /// Result containing the parsed ComplexityLevel or error
    ///
    /// # Examples
    ///
    /// ```rust
    /// let level = ComplexityLevel::from_str("minimal").unwrap();
    /// assert_eq!(level, ComplexityLevel::Minimal);
    /// ```
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(match value.trim().to_lowercase().as_str() {
            "minimal" => ComplexityLevel::Minimal,
            _ => ComplexityLevel::Comprehensive,
        })
    }
}

impl fmt::Display for ComplexityLevel {
    /// Format the complexity level for display.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter to write to
    ///
    /// # Returns
    ///
    /// Result of the formatting operation
    ///
    /// # Examples
    ///
    /// ```rust
    /// let level = ComplexityLevel::Comprehensive;
    /// println!("{}", level); // Prints "comprehensive"
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct GenerationConfig {
    /// Complexity level guiding generated specs.
    pub complexity: ComplexityLevel,
    /// Minimum features for comprehensive specs.
    pub min_features: u32,
    /// Minimum database tables for comprehensive specs.
    pub min_database_tables: u32,
    /// Minimum API endpoints for comprehensive specs.
    pub min_api_endpoints: u32,
    /// Minimum implementation steps for comprehensive specs.
    pub min_implementation_steps: u32,
    /// Minimum features for minimal specs.
    pub minimal_min_features: u32,
    /// Minimum database tables for minimal specs.
    pub minimal_min_database_tables: u32,
    /// Minimum API endpoints for minimal specs.
    pub minimal_min_api_endpoints: u32,
    /// Minimum implementation steps for minimal specs.
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
    /// Enable parallel subagent spec generation (faster but uses more tokens)
    pub enable_subagents: bool,
}

/// Minimum content requirements derived from generation settings.
#[derive(Debug, Clone, Copy)]
pub struct GenerationRequirements {
    /// Minimum number of core features.
    pub min_features: u32,
    /// Minimum number of database tables.
    pub min_database_tables: u32,
    /// Minimum number of API endpoints.
    pub min_api_endpoints: u32,
    /// Minimum number of implementation steps.
    pub min_implementation_steps: u32,
}

impl GenerationConfig {
    /// Get minimum requirements based on the selected complexity level.
    ///
    /// # Returns
    ///
    /// GenerationRequirements struct containing minimum counts for various spec elements
    ///
    /// # Examples
    ///
    /// ```rust
    /// let config = GenerationConfig::default();
    /// let requirements = config.requirements();
    /// println!("Minimum features: {}", requirements.min_features);
    /// ```
    pub fn requirements(&self) -> GenerationRequirements {
        if self.complexity == ComplexityLevel::Minimal {
            GenerationRequirements {
                min_features: self.minimal_min_features,
                min_database_tables: self.minimal_min_database_tables,
                min_api_endpoints: self.minimal_min_api_endpoints,
                min_implementation_steps: self.minimal_min_implementation_steps,
            }
        } else {
            GenerationRequirements {
                min_features: self.min_features,
                min_database_tables: self.min_database_tables,
                min_api_endpoints: self.min_api_endpoints,
                min_implementation_steps: self.min_implementation_steps,
            }
        }
    }
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            complexity: ComplexityLevel::default(),
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
