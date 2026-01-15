//! IPC protocol types matching the Go client definitions.
//!
//! Messages are newline-delimited JSON (NDJSON) with a standard envelope.

#![allow(dead_code)] // Protocol types are defined for completeness

use serde::{Deserialize, Serialize};

/// Protocol version for compatibility checking.
pub const PROTOCOL_VERSION: &str = "1.0.0";

/// Direction of message flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    #[serde(rename = "rust->go")]
    RustToGo,
    #[serde(rename = "go->rust")]
    GoToRust,
}

/// Type of message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageType {
    Event,
    Command,
}

/// Envelope for all IPC messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub protocol_version: String,
    pub direction: Direction,
    #[serde(rename = "type")]
    pub msg_type: MessageType,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,
}

impl Message {
    /// Create a new event message from Rust to Go.
    pub fn event(name: impl Into<String>, payload: impl Serialize) -> anyhow::Result<Self> {
        Ok(Self {
            protocol_version: PROTOCOL_VERSION.to_string(),
            direction: Direction::RustToGo,
            msg_type: MessageType::Event,
            name: name.into(),
            payload: Some(serde_json::to_value(payload)?),
        })
    }

    /// Create an event message without a payload.
    pub fn event_empty(name: impl Into<String>) -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION.to_string(),
            direction: Direction::RustToGo,
            msg_type: MessageType::Event,
            name: name.into(),
            payload: None,
        }
    }
}

// --- Event Names ---

/// Event name constants for Rust -> Go messages.
pub mod events {
    pub const ENGINE_READY: &str = "EngineReady";
    pub const LOG_LINE: &str = "LogLine";
    pub const PROGRESS_UPDATE: &str = "ProgressUpdate";
    pub const USER_PROMPT: &str = "UserPrompt";
    pub const FINISHED: &str = "Finished";
    pub const ERROR: &str = "Error";
    pub const MODE_LIST: &str = "ModeList";
    pub const CONFIG_LOADED: &str = "ConfigLoaded";
}

/// Command name constants for Go -> Rust messages.
pub mod commands {
    pub const START_VIBE: &str = "StartVibe";
    pub const HANDLE_SELECTION: &str = "HandleSelection";
    pub const CANCEL: &str = "Cancel";
    pub const OPEN_CONFIG: &str = "OpenConfig";
    pub const RETRY: &str = "Retry";
    pub const SELECT_MODE: &str = "SelectMode";
    pub const CONFIRM: &str = "Confirm";
}

// --- Event Payloads (Rust -> Go) ---

/// Sent when the Rust engine is ready.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineReadyPayload {
    pub version: String,
    pub work_dir: String,
}

/// A log message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogLinePayload {
    pub level: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

/// Progress update.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressUpdatePayload {
    pub phase: String,
    pub current: usize,
    pub total: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub percentage: Option<f64>,
}

/// User prompt for input/selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPromptPayload {
    pub prompt_id: String,
    pub prompt_type: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_cancel: Option<bool>,
}

/// Completion notice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinishedPayload {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Error report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fatal: Option<bool>,
}

/// Available modes list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeListPayload {
    pub modes: Vec<ModeInfo>,
}

/// Mode information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeInfo {
    pub id: String,
    pub label: String,
    pub description: String,
}

/// Configuration loaded status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigLoadedPayload {
    pub has_existing_config: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_path: Option<String>,
}

// --- Command Payloads (Go -> Rust) ---

/// Mode selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectModePayload {
    pub mode_id: String,
}

/// Response to a user prompt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandleSelectionPayload {
    pub prompt_id: String,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<usize>,
}

/// Confirmation response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmPayload {
    pub prompt_id: String,
    pub confirmed: bool,
}

/// Cancel request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Start vibe request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartVibePayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub single_model: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel: Option<usize>,
}
