//! Conductor-style context-driven development
//!
//! Implements the three-phase workflow:
//! 1. Context setup (product/tech/workflow)
//! 2. Spec + Plan generation per track
//! 3. Implementation with checkpoints
//!
//! This module integrates with the existing feature_list.json workflow,
//! adding persistent planning artifacts that survive across sessions.

pub mod plan;

use anyhow::Result;
use std::fs;
use std::path::Path;

use crate::config::Config;

// Re-export plan types for convenience
pub use plan::{get_next_task, mark_task_complete, parse_plan};

// ─────────────────────────────────────────────────────────────────────────────
// Context Management
// ─────────────────────────────────────────────────────────────────────────────

/// Check if conductor context has been set up
pub fn context_exists(config: &Config) -> bool {
    let context_dir = Path::new(&config.conductor.context_dir);
    context_dir.exists() && context_dir.join("product.md").exists()
}

// ─────────────────────────────────────────────────────────────────────────────
// Track Management
// ─────────────────────────────────────────────────────────────────────────────

/// A track represents a unit of work (typically one feature)
#[derive(Debug, Clone)]
pub struct Track {
    /// Slug name of the track (directory name)
    #[allow(dead_code)]
    pub name: String,
    /// Path to the track directory
    pub path: std::path::PathBuf,
}

/// Get the active track (if any) - the one with incomplete tasks
pub fn get_active_track(config: &Config) -> Result<Option<Track>> {
    let tracks_dir = Path::new(&config.conductor.tracks_dir);

    if !tracks_dir.exists() {
        return Ok(None);
    }

    for entry in fs::read_dir(tracks_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let plan_path = path.join("plan.md");
            if plan_path.exists() {
                let plan_content = fs::read_to_string(&plan_path)?;
                // Check if there are unchecked tasks
                if plan_content.contains("- [ ]") {
                    return Ok(Some(Track {
                        name: entry.file_name().to_string_lossy().to_string(),
                        path: path.clone(),
                    }));
                }
            }
        }
    }

    Ok(None)
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_plan() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let plan_path = temp_dir.path().join("plan.md");

        let content = r#"# Plan: Test Feature

## Phase 1: Setup

- [ ] Task 1.1: Create files
  - [ ] Subtask 1.1.1: Create src/
  - [x] Subtask 1.1.2: Create tests/
- [x] Task 1.2: Configure

## Phase 2: Implementation

- [ ] Task 2.1: Build core
"#;
        fs::write(&plan_path, content).unwrap();

        let tasks = parse_plan(&plan_path).unwrap();
        assert_eq!(tasks.len(), 5);

        // Check first task
        assert_eq!(tasks[0].description, "Task 1.1: Create files");
        assert!(!tasks[0].complete);
        assert_eq!(tasks[0].level, 0);

        // Check subtask
        assert_eq!(tasks[1].description, "Subtask 1.1.1: Create src/");
        assert!(!tasks[1].complete);
        assert_eq!(tasks[1].level, 1);

        // Check completed subtask
        assert_eq!(tasks[2].description, "Subtask 1.1.2: Create tests/");
        assert!(tasks[2].complete);

        // Check next incomplete task
        let next = get_next_task(&tasks).unwrap();
        assert_eq!(next.description, "Task 1.1: Create files");
    }

    #[test]
    fn test_mark_task_complete() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let plan_path = temp_dir.path().join("plan.md");

        let content = "- [ ] Task 1\n- [ ] Task 2\n- [x] Task 3\n";
        fs::write(&plan_path, content).unwrap();

        mark_task_complete(&plan_path, 1).unwrap();

        let updated = fs::read_to_string(&plan_path).unwrap();
        assert!(updated.contains("- [x] Task 1"));
        assert!(updated.contains("- [ ] Task 2"));
    }
}
