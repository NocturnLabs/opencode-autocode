use serde::Deserialize;

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
    /// Idle timeout in seconds (0 = no timeout)
    pub idle_timeout_seconds: u32,
    /// Auto-commit after feature completion
    pub auto_commit: bool,
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
// Conductor Configuration (Context-Driven Planning)
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ConductorConfig {
    /// Directory for project context files
    pub context_dir: String,
    /// Directory for track-based work units (per-feature specs/plans)
    pub tracks_dir: String,
    /// Auto-generate context files on first run
    pub auto_setup: bool,
    /// Planning mode: "auto" (AI generates) or "manual" (user writes)
    pub planning_mode: String,
    /// Checkpoint frequency: save progress after N completed tasks
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
