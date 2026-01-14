use serde::Deserialize;

// ─────────────────────────────────────────────────────────────────────────────
// Autonomous Session Configuration
// ─────────────────────────────────────────────────────────────────────────────

/// Configuration for autonomous coding sessions
///
/// Controls the behavior of automated coding sessions where the AI works independently.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AutonomousConfig {
    /// Seconds between autonomous sessions
    ///
    /// Controls how frequently autonomous sessions are initiated.
    pub delay_between_sessions: u32,

    /// Maximum iterations (0 = unlimited)
    ///
    /// Limits the number of iterations an autonomous session can perform.
    pub max_iterations: u32,

    /// Log level for opencode commands
    ///
    /// Controls the verbosity of logging during autonomous sessions.
    pub log_level: String,

    /// Session timeout in minutes (0 = no timeout)
    ///
    /// Maximum duration for an autonomous session.
    pub session_timeout_minutes: u32,

    /// Idle timeout in seconds (0 = no timeout)
    ///
    /// Maximum time without activity before session is considered idle.
    pub idle_timeout_seconds: u32,

    /// Auto-commit after feature completion
    ///
    /// Whether to automatically commit changes after completing a feature.
    pub auto_commit: bool,

    /// Number of iterations without progress before warning (0 = unlimited)
    ///
    /// Triggers warnings when the AI appears to be stuck.
    pub max_no_progress: u32,
}

impl Default for AutonomousConfig {
    fn default() -> Self {
        Self {
            delay_between_sessions: 5,
            max_iterations: 0,
            log_level: "DEBUG".to_string(),
            session_timeout_minutes: 15,
            idle_timeout_seconds: 600,
            auto_commit: true,
            max_no_progress: 5,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Agent Behavior Configuration
// ─────────────────────────────────────────────────────────────────────────────

/// Configuration for AI agent behavior
///
/// Controls how the AI agent handles tasks, retries, and verification.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AgentConfig {
    /// Max retry attempts before research protocol
    ///
    /// Number of times to retry a failed task before switching to research mode.
    pub max_retry_attempts: u32,

    /// Max research-based attempts before moving on
    ///
    /// Number of research attempts before giving up on a task.
    pub max_research_attempts: u32,

    /// Number of passing features to verify before new work
    ///
    /// How many features to verify are working before starting new tasks.
    pub verification_sample_size: u32,

    /// Focus on one feature at a time
    ///
    /// Whether to complete one feature fully before moving to the next.
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

/// Configuration for alternative approach generation when the AI gets stuck
///
/// When the AI encounters persistent failures, this configuration controls
/// whether and how to generate alternative approaches to solve the problem.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AlternativeApproachesConfig {
    /// Enable alternative approach generation when stuck
    ///
    /// Whether to generate alternative approaches when the AI is stuck.
    pub enabled: bool,

    /// Number of alternative approaches to generate
    ///
    /// How many different approaches to generate for each stuck scenario.
    pub num_approaches: u32,

    /// Retry threshold before triggering alternative generation
    ///
    /// Number of failed attempts before generating alternative approaches.
    pub retry_threshold: u32,

    /// Cache results to avoid regenerating
    ///
    /// Whether to cache generated approaches to avoid redundant work.
    pub cache_results: bool,

    /// Cache directory
    ///
    /// Directory where cached alternative approaches are stored.
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
// Conductor Configuration (Context-Driven Planning)
// ─────────────────────────────────────────────────────────────────────────────

/// Configuration for the conductor system that manages context and planning
///
/// The conductor maintains project context and manages track-based work units
/// for organized feature development.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ConductorConfig {
    /// Directory for project context files
    ///
    /// Where project context and metadata are stored.
    pub context_dir: String,

    /// Directory for track-based work units (per-feature specs/plans)
    ///
    /// Where individual feature specifications and plans are stored.
    pub tracks_dir: String,

    /// Auto-generate context files on first run
    ///
    /// Whether to automatically set up context files when first running.
    pub auto_setup: bool,

    /// Planning mode: "auto" (AI generates) or "manual" (user writes)
    ///
    /// Controls whether planning is done automatically by AI or manually by user.
    pub planning_mode: String,

    /// Checkpoint frequency: save progress after N completed tasks
    ///
    /// How often to save progress checkpoints.
    pub checkpoint_frequency: u32,
}

impl Default for ConductorConfig {
    fn default() -> Self {
        Self {
            context_dir: ".conductor".to_string(),
            tracks_dir: "tracks".to_string(),
            auto_setup: true,
            planning_mode: "auto".to_string(),
            checkpoint_frequency: 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alternative_approaches_defaults() {
        let aa = AlternativeApproachesConfig::default();
        assert_eq!(aa.num_approaches, 7);
        assert_eq!(aa.retry_threshold, 3);
        assert!(aa.cache_results);
    }
}
