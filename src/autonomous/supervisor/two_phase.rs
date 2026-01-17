//! Two-phase orchestration for feature implementation
//!
//! This module implements reasoning → coding workflow where:
//! - Reasoning phase: Expensive model produces structured implementation packet
//! - Coding phase: Autonomous model executes packet

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs;
use std::path::Path;

use crate::autonomous::session;
use crate::autonomous::settings;
use crate::common::logging::DebugLogger;
use crate::config::Config;
use crate::db::features::Feature;

/// Represents a file action in the implementation packet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAction {
    /// Path to the file
    pub path: String,
    /// Type of action to perform
    pub action: FileActionType,
}

/// Types of file actions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileActionType {
    Create,
    Modify,
    Delete,
}

/// Represents a code edit in the implementation packet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edit {
    /// File path to edit
    pub file: String,
    /// Description of the edit
    pub description: String,
    /// Code to apply (can be full content or partial)
    pub code: String,
}

/// Represents a command to run during implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    /// Command to execute
    pub command: String,
    /// Description of what the command does
    pub description: String,
}

/// Implementation packet - structured JSON handoff from reasoning to coding phase
///
/// This is output of the reasoning phase and input to the coding phase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationPacket {
    /// ID of the feature being implemented
    pub feature_id: i64,
    /// Human-readable feature description
    pub feature_description: String,
    /// Files that need to be created, modified, or deleted
    pub files_to_modify: Vec<FileAction>,
    /// Specific edits to apply to files
    pub edits: Vec<Edit>,
    /// Commands to run during implementation
    pub commands_to_run: Vec<Command>,
    /// Command to verify the feature works
    pub verification_command: String,
}

impl ImplementationPacket {
    /// Validates that the implementation packet has all required fields
    pub fn validate(&self) -> Result<(), String> {
        if self.feature_id <= 0 {
            return Err("feature_id must be positive".to_string());
        }
        if self.feature_description.trim().is_empty() {
            return Err("feature_description cannot be empty".to_string());
        }
        if self.verification_command.trim().is_empty() {
            return Err("verification_command cannot be empty".to_string());
        }
        Ok(())
    }
}

/// Result from reasoning phase
pub enum ReasoningResult {
    /// Successfully produced a valid implementation packet
    Success(ImplementationPacket),
    /// Failed to produce valid JSON
    InvalidJson(String),
    /// JSON doesn't match schema or validation fails
    ValidationError(String),
    /// Session error (timeout, crash, etc.)
    Error(String),
}

/// Execute reasoning phase to produce implementation packet
///
/// This phase uses reasoning model to analyze feature and produce
/// a structured JSON implementation packet.
pub fn execute_reasoning_phase(
    feature: &Feature,
    config: &Config,
    settings: &settings::LoopSettings,
    logger: &DebugLogger,
) -> Result<ReasoningResult> {
    logger.info(&format!(
        "Starting reasoning phase for feature #{}: {}",
        feature.id.unwrap_or(0),
        feature.description
    ));

    // Generate reasoning phase prompt
    let prompt = generate_reasoning_prompt(feature, config)?;

    // Write to temp file for opencode
    let prompt_path = Path::new(".opencode/command/reasoning-phase.md");
    fs::create_dir_all(prompt_path.parent().unwrap())
        .context("Failed to create command directory")?;
    fs::write(prompt_path, &prompt).context("Failed to write reasoning prompt")?;

    // Execute reasoning session with reasoning model
    let session_options = session::SessionOptions {
        command: "reasoning-phase".to_string(),
        model: settings.reasoning_model.clone(),
        log_level: settings.log_level.clone(),
        session_id: None,
        timeout_minutes: settings.session_timeout,
        idle_timeout_seconds: settings.idle_timeout,
        opencode_path: settings.opencode_path.clone(),
    };

    let result = session::execute_opencode_session(session_options, logger);

    match result {
        Ok(session::SessionResult::Continue) => {
            // Try to parse output as JSON
            let output_path = Path::new(".opencode/reasoning-output.json");
            if output_path.exists() {
                let json_str =
                    fs::read_to_string(output_path).context("Failed to read reasoning output")?;
                match serde_json::from_str::<ImplementationPacket>(&json_str) {
                    Ok(mut packet) => {
                        // Verify packet matches expected feature
                        if packet.feature_id != feature.id.unwrap_or(0) {
                            return Ok(ReasoningResult::ValidationError(format!(
                                "Packet feature_id {} doesn't match feature {}",
                                packet.feature_id,
                                feature.id.unwrap_or(0)
                            )));
                        }

                        // Validate packet content
                        if let Err(e) = packet.validate() {
                            return Ok(ReasoningResult::ValidationError(e));
                        }

                        // Update description from feature if empty
                        if packet.feature_description.is_empty() {
                            packet.feature_description = feature.description.clone();
                        }

                        logger.info("Reasoning phase produced valid implementation packet");
                        Ok(ReasoningResult::Success(packet))
                    }
                    Err(e) => {
                        let error_msg = format!("Failed to parse JSON: {}", e);
                        logger.error(&error_msg);
                        Ok(ReasoningResult::InvalidJson(error_msg))
                    }
                }
            } else {
                // No JSON output - check if model put JSON in session output
                Ok(ReasoningResult::InvalidJson(
                    "No JSON output file found".to_string(),
                ))
            }
        }
        Ok(session::SessionResult::Error(msg)) => {
            logger.error(&format!("Reasoning phase error: {}", msg));
            Ok(ReasoningResult::Error(msg))
        }
        Ok(session::SessionResult::EarlyTerminated { trigger }) => {
            logger.warning(&format!("Reasoning phase terminated early: {}", trigger));
            Ok(ReasoningResult::Error(format!(
                "Early terminated: {}",
                trigger
            )))
        }
        Ok(session::SessionResult::Stopped) => Ok(ReasoningResult::Error("Stopped".to_string())),
        Err(e) => {
            logger.error(&format!("Reasoning phase execution error: {}", e));
            Ok(ReasoningResult::Error(e.to_string()))
        }
    }
}

/// Execute coding phase to implement feature based on packet
///
/// This phase uses the autonomous/coding model to execute the edits
/// and commands from the implementation packet.
pub fn execute_coding_phase(
    packet: &ImplementationPacket,
    _feature: &Feature,
    settings: &settings::LoopSettings,
    logger: &DebugLogger,
) -> Result<session::SessionResult> {
    logger.info(&format!(
        "Starting coding phase for feature #{}: {}",
        packet.feature_id, packet.feature_description
    ));

    // Generate coding phase prompt with packet embedded
    let prompt = generate_coding_prompt(packet)?;

    // Write to temp file for opencode
    let prompt_path = Path::new(".opencode/command/coding-phase.md");
    fs::create_dir_all(prompt_path.parent().unwrap())
        .context("Failed to create command directory")?;
    fs::write(prompt_path, &prompt).context("Failed to write coding prompt")?;

    // Execute coding session with autonomous model
    let session_options = session::SessionOptions {
        command: "coding-phase".to_string(),
        model: settings.coding_model.clone(),
        log_level: settings.log_level.clone(),
        session_id: None,
        timeout_minutes: settings.session_timeout,
        idle_timeout_seconds: settings.idle_timeout,
        opencode_path: settings.opencode_path.clone(),
    };

    session::execute_opencode_session(session_options, logger)
}

/// Generate prompt for reasoning phase
fn generate_reasoning_prompt(feature: &Feature, config: &Config) -> Result<String> {
    let steps_text = if feature.steps.is_empty() {
        "Not specified - create comprehensive implementation plan".to_string()
    } else {
        feature
            .steps
            .iter()
            .enumerate()
            .map(|(i, s)| format!("{}. {}", i + 1, s))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let prompt = format!(
        r#"# Reasoning Phase: Implementation Planning

## Your Task
You are in **reasoning phase** of a two-phase orchestration workflow. Your job is to produce a structured implementation plan for the following feature. Do NOT write any code or execute any commands - only plan the implementation.

## Feature Details

**Feature #{}**: {}

## Acceptance Criteria

{}

## Context
- App spec file: `{}`

## Instructions

1. Analyze feature requirements and acceptance criteria
2. Determine which files need to be created, modified, or deleted
3. Plan specific code edits with clear descriptions
4. Identify commands needed to build/test/verify the implementation
5. Output ONLY a valid JSON implementation packet

## JSON Schema

You MUST output a valid JSON object with this exact structure:

```json
{{
  "feature_id": {},
  "feature_description": "{}",
  "files_to_modify": [
    {{
      "path": "path/to/file.ext",
      "action": "create|modify|delete"
    }}
  ],
  "edits": [
    {{
      "file": "path/to/file.ext",
      "description": "Clear description of the edit",
      "code": "The actual code to apply (or description of change)"
    }}
  ],
  "commands_to_run": [
    {{
      "command": "command to run",
      "description": "What this command does"
    }}
  ],
  "verification_command": "command to verify the feature works"
}}
```

## Important Rules

- Output ONLY the JSON - no other text, no markdown code blocks
- Be specific with file paths and code descriptions
- Include a verification command that proves the feature works
- Consider test files, build commands, and documentation updates
- Make the plan comprehensive but focused

After you output the JSON, the coding phase will execute your plan.
"#,
        feature.id.unwrap_or(0),
        feature.description,
        steps_text,
        config.paths.app_spec_file,
        feature.id.unwrap_or(0),
        feature.description
    );

    Ok(prompt)
}

/// Generate prompt for coding phase
fn generate_coding_prompt(packet: &ImplementationPacket) -> Result<String> {
    let mut prompt = format!(
        r#"# Coding Phase: Feature Implementation

## Your Task
You are in **coding phase** of a two-phase orchestration workflow. Your job is to execute the implementation plan provided below. Follow the plan precisely.

## Feature Details

**Feature #{}**: {}

## Implementation Plan

The reasoning phase has produced this implementation plan:

### Files to Modify
"#,
        packet.feature_id, packet.feature_description
    );

    for file_action in &packet.files_to_modify {
        prompt.push_str(&format!(
            "\n- **{}**: `{}`",
            match &file_action.action {
                FileActionType::Create => "CREATE",
                FileActionType::Modify => "MODIFY",
                FileActionType::Delete => "DELETE",
            },
            file_action.path
        ));
    }

    prompt.push_str("\n\n### Edits to Apply\n");
    for edit in &packet.edits {
        prompt.push_str(&format!(
            "\n- **{}**\n  - File: `{}`\n  - Description: {}\n",
            file!(),
            edit.file,
            edit.description
        ));
    }

    prompt.push_str("\n\n### Commands to Run\n");
    for cmd in &packet.commands_to_run {
        prompt.push_str(&format!("- `{}` - {}\n", cmd.command, cmd.description));
    }

    prompt.push_str(&format!(
        r#"
### Verification Command
```
{}
```

## Instructions

1. Read project context to understand the codebase
2. Apply each edit in the order specified
3. Run each command in the order specified
4. Verify the feature works using the verification command
5. Output `===SESSION_COMPLETE===` when you are done

## Rules

- Follow the implementation plan precisely
- If you need to deviate, explain why in comments
- You may run additional commands for testing/debugging
- Do NOT run git commands (the supervisor will commit)
- **ALWAYS** output `===SESSION_COMPLETE===` when the implementation is complete

The supervisor will:
- Run the verification command
- Commit changes if the verification passes
- Mark the feature as complete
"#,
        packet.verification_command
    ));

    Ok(prompt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_implementation_packet_serialization() {
        let packet = ImplementationPacket {
            feature_id: 1,
            feature_description: "Add user authentication".to_string(),
            files_to_modify: vec![
                FileAction {
                    path: "src/auth.rs".to_string(),
                    action: FileActionType::Create,
                },
                FileAction {
                    path: "src/main.rs".to_string(),
                    action: FileActionType::Modify,
                },
            ],
            edits: vec![Edit {
                file: "src/auth.rs".to_string(),
                description: "Create auth module".to_string(),
                code: "pub fn authenticate() {}".to_string(),
            }],
            commands_to_run: vec![Command {
                command: "cargo build".to_string(),
                description: "Build the project".to_string(),
            }],
            verification_command: "cargo test".to_string(),
        };

        // Test serialization
        let json = serde_json::to_string(&packet).expect("Failed to serialize");
        assert!(json.contains("\"feature_id\":1"));
        assert!(json.contains("\"feature_description\":\"Add user authentication\""));

        // Test deserialization
        let deserialized: ImplementationPacket =
            serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(deserialized.feature_id, 1);
        assert_eq!(deserialized.files_to_modify.len(), 2);
        assert_eq!(deserialized.edits.len(), 1);
        assert_eq!(deserialized.commands_to_run.len(), 1);
    }

    #[test]
    fn test_implementation_packet_validation_success() {
        let packet = ImplementationPacket {
            feature_id: 1,
            feature_description: "Valid feature".to_string(),
            files_to_modify: vec![],
            edits: vec![],
            commands_to_run: vec![],
            verification_command: "cargo test".to_string(),
        };

        assert!(packet.validate().is_ok());
    }

    #[test]
    fn test_implementation_packet_validation_invalid_id() {
        let packet = ImplementationPacket {
            feature_id: 0,
            feature_description: "Valid feature".to_string(),
            files_to_modify: vec![],
            edits: vec![],
            commands_to_run: vec![],
            verification_command: "cargo test".to_string(),
        };

        let result = packet.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("feature_id must be positive"));
    }

    #[test]
    fn test_implementation_packet_validation_empty_description() {
        let packet = ImplementationPacket {
            feature_id: 1,
            feature_description: "   ".to_string(),
            files_to_modify: vec![],
            edits: vec![],
            commands_to_run: vec![],
            verification_command: "cargo test".to_string(),
        };

        let result = packet.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("feature_description cannot be empty"));
    }

    #[test]
    fn test_implementation_packet_validation_empty_verification() {
        let packet = ImplementationPacket {
            feature_id: 1,
            feature_description: "Valid feature".to_string(),
            files_to_modify: vec![],
            edits: vec![],
            commands_to_run: vec![],
            verification_command: "".to_string(),
        };

        let result = packet.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("verification_command cannot be empty"));
    }

    #[test]
    fn test_file_action_type_serialization() {
        let create = FileAction {
            path: "test.rs".to_string(),
            action: FileActionType::Create,
        };
        let modify = FileAction {
            path: "test.rs".to_string(),
            action: FileActionType::Modify,
        };
        let delete = FileAction {
            path: "test.rs".to_string(),
            action: FileActionType::Delete,
        };

        let create_json = serde_json::to_string(&create).unwrap();
        let modify_json = serde_json::to_string(&modify).unwrap();
        let delete_json = serde_json::to_string(&delete).unwrap();

        assert!(create_json.contains("\"action\":\"create\""));
        assert!(modify_json.contains("\"action\":\"modify\""));
        assert!(delete_json.contains("\"action\":\"delete\""));
    }

    #[test]
    fn test_json_parsing_from_string() {
        let json_str = r#"{
            "feature_id": 42,
            "feature_description": "Test feature",
            "files_to_modify": [
                {"path": "src/lib.rs", "action": "modify"}
            ],
            "edits": [
                {"file": "src/lib.rs", "description": "Add function", "code": "fn foo() {}"}
            ],
            "commands_to_run": [
                {"command": "cargo check", "description": "Verify compilation"}
            ],
            "verification_command": "cargo test"
        }"#;

        let packet: ImplementationPacket =
            serde_json::from_str(json_str).expect("Failed to parse JSON");
        assert_eq!(packet.feature_id, 42);
        assert_eq!(packet.feature_description, "Test feature");
        assert_eq!(packet.files_to_modify.len(), 1);
        assert!(matches!(
            packet.files_to_modify[0].action,
            FileActionType::Modify
        ));
        assert_eq!(packet.edits.len(), 1);
        assert_eq!(packet.commands_to_run.len(), 1);
        assert_eq!(packet.verification_command, "cargo test");
    }
}
