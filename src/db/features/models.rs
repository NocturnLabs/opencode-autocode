use serde::{Deserialize, Serialize};

/// Feature data structure (matches the old JSON format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    /// Database ID (optional for JSON import)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,

    /// Feature category (functional, style, integration, performance)
    pub category: String,

    /// Human-readable description
    pub description: String,

    /// Verification steps
    pub steps: Vec<String>,

    /// Whether this feature passes all tests
    pub passes: bool,

    /// Optional shell command for automated verification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_command: Option<String>,

    /// Last verification error (for auto-fix context)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
}
