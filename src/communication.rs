//! Agent-User Communication Channel
//!
//! Provides a polling-based communication channel between the autonomous agent
//! and the user via a markdown file (COMMUNICATION.md).

// Allow unused public APIs until full integration with autonomous runner
#![allow(dead_code)]

use anyhow::{Context, Result};
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Default path for communication file
pub const DEFAULT_COMMUNICATION_PATH: &str = ".autocode/COMMUNICATION.md";

/// Question status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QuestionStatus {
    Pending,
    Resolved,
}

impl std::fmt::Display for QuestionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QuestionStatus::Pending => write!(f, "⏳ PENDING"),
            QuestionStatus::Resolved => write!(f, "✅ RESOLVED"),
        }
    }
}

/// A question posted by the agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    pub id: String,
    pub session: i32,
    pub timestamp: String,
    pub title: String,
    pub body: String,
    pub options: Vec<String>,
    pub status: QuestionStatus,
}

/// A response from the user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub question_id: String,
    pub timestamp: String,
    pub body: String,
}

/// Communication channel for agent-user interaction
pub struct CommunicationChannel {
    path: PathBuf,
}

impl CommunicationChannel {
    /// Create a new communication channel
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
        }
    }

    /// Check if communication file exists
    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    /// Initialize communication file with template
    pub fn init(&self) -> Result<()> {
        if self.exists() {
            return Ok(());
        }

        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let template = r#"# Agent-User Communication

This file enables communication between you and the autonomous agent during `vibe` sessions.

## How to Use

1. **Agent Questions** appear below when the agent needs guidance
2. **Your Responses** go in the "User Responses" section
3. The agent checks this file at the start of each session

---

## Agent Questions

<!-- Questions from the agent will appear here -->

---

## User Responses

<!-- Add your responses here, referencing the question title -->

---

## Resolved Questions

<!-- Resolved questions are moved here automatically -->
"#;

        fs::write(&self.path, template)
            .with_context(|| format!("Failed to create {}", self.path.display()))?;

        Ok(())
    }

    /// Post a new question from the agent
    pub fn post_question(
        &self,
        session: i32,
        title: &str,
        body: &str,
        options: &[&str],
    ) -> Result<String> {
        let content = self.read_file()?;
        let id = format!("q-{}-{}", session, Local::now().format("%H%M%S"));
        let timestamp = Local::now().format("%Y-%m-%d %H:%M");

        let mut options_text = String::new();
        for (i, opt) in options.iter().enumerate() {
            options_text.push_str(&format!("{}. {}\n", i + 1, opt));
        }

        let question_block = format!(
            r#"
### [{timestamp}] Session #{session} - {title}
**Status:** ⏳ PENDING
**ID:** `{id}`

{body}

{options_text}
---
"#
        );

        // Insert after "## Agent Questions" marker
        let marker = "## Agent Questions";
        let new_content = if let Some(pos) = content.find(marker) {
            let insert_pos = pos + marker.len();
            let (before, after) = content.split_at(insert_pos);
            format!("{}\n{}{}", before, question_block, after)
        } else {
            // If marker not found, append to end
            format!("{}\n{}", content, question_block)
        };

        fs::write(&self.path, new_content)?;
        Ok(id)
    }

    /// Check for new user responses
    pub fn check_responses(&self) -> Result<Vec<Response>> {
        let content = self.read_file()?;
        let mut responses = Vec::new();

        // Find "## User Responses" section
        let responses_marker = "## User Responses";
        let resolved_marker = "## Resolved Questions";

        if let Some(start) = content.find(responses_marker) {
            let end = content.find(resolved_marker).unwrap_or(content.len());
            let section = &content[start..end];

            // Parse responses (format: ### [timestamp] RE: question-title)
            for line in section.lines() {
                if line.starts_with("### ") && line.contains("RE:") {
                    // Extract the question reference and find the response body
                    if let Some(re_pos) = line.find("RE:") {
                        let question_ref = line[re_pos + 3..].trim();
                        // For now, capture everything until next ### or section marker
                        responses.push(Response {
                            question_id: question_ref.to_string(),
                            timestamp: Local::now().format("%Y-%m-%d %H:%M").to_string(),
                            body: String::new(), // Would need more parsing for full body
                        });
                    }
                }
            }
        }

        Ok(responses)
    }

    /// Get all pending questions
    pub fn get_pending_questions(&self) -> Result<Vec<Question>> {
        let content = self.read_file()?;
        let mut questions = Vec::new();

        // Find questions with PENDING status
        let questions_marker = "## Agent Questions";
        let responses_marker = "## User Responses";

        if let Some(start) = content.find(questions_marker) {
            let end = content.find(responses_marker).unwrap_or(content.len());
            let section = &content[start..end];

            // Simple check for pending status
            if section.contains("⏳ PENDING") {
                // Count pending questions
                for line in section.lines() {
                    if line.contains("⏳ PENDING") {
                        questions.push(Question {
                            id: String::new(),
                            session: 0,
                            timestamp: Local::now().format("%Y-%m-%d %H:%M").to_string(),
                            title: "Pending question".to_string(),
                            body: String::new(),
                            options: vec![],
                            status: QuestionStatus::Pending,
                        });
                    }
                }
            }
        }

        Ok(questions)
    }

    /// Mark a question as resolved
    pub fn mark_resolved(&self, question_id: &str) -> Result<bool> {
        let content = self.read_file()?;

        // Replace PENDING with RESOLVED for this question
        let id_marker = format!("**ID:** `{}`", question_id);
        if !content.contains(&id_marker) {
            return Ok(false);
        }

        let new_content = content.replace(
            &format!("{}\n\n⏳ PENDING", id_marker),
            &format!("{}\n\n✅ RESOLVED", id_marker),
        );

        // Also try alternative format
        let new_content = new_content.replace("⏳ PENDING", "✅ RESOLVED");

        fs::write(&self.path, new_content)?;
        Ok(true)
    }

    /// Read the communication file
    fn read_file(&self) -> Result<String> {
        if !self.exists() {
            self.init()?;
        }
        fs::read_to_string(&self.path)
            .with_context(|| format!("Failed to read {}", self.path.display()))
    }

    /// Check if there are any pending questions
    pub fn has_pending_questions(&self) -> Result<bool> {
        let questions = self.get_pending_questions()?;
        Ok(!questions.is_empty())
    }

    /// Check if there are any unread responses
    pub fn has_unread_responses(&self) -> Result<bool> {
        let responses = self.check_responses()?;
        Ok(!responses.is_empty())
    }
}

impl Default for CommunicationChannel {
    fn default() -> Self {
        Self::new(Path::new(DEFAULT_COMMUNICATION_PATH))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_init_creates_file() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("COMMUNICATION.md");
        let channel = CommunicationChannel::new(&path);

        assert!(!channel.exists());
        channel.init().unwrap();
        assert!(channel.exists());
    }

    #[test]
    fn test_post_question() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("COMMUNICATION.md");
        let channel = CommunicationChannel::new(&path);

        channel.init().unwrap();
        let id = channel
            .post_question(
                1,
                "Build Error",
                "The build is failing. What should I do?",
                &["Fix the error", "Skip this feature", "Ask for help"],
            )
            .unwrap();

        assert!(id.starts_with("q-1-"));

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("Build Error"));
        assert!(content.contains("⏳ PENDING"));
    }

    #[test]
    fn test_has_pending_questions() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("COMMUNICATION.md");
        let channel = CommunicationChannel::new(&path);

        channel.init().unwrap();
        assert!(!channel.has_pending_questions().unwrap());

        channel
            .post_question(1, "Test", "Test body", &[])
            .unwrap();
        assert!(channel.has_pending_questions().unwrap());
    }
}
