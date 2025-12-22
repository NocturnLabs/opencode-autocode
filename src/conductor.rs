//! Conductor-style context-driven development
//!
//! Implements the three-phase workflow:
//! 1. Context setup (product/tech/workflow)
//! 2. Spec + Plan generation per track
//! 3. Implementation with checkpoints
//!
//! This module integrates with the existing feature_list.json workflow,
//! adding persistent planning artifacts that survive across sessions.

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::config::Config;

// ─────────────────────────────────────────────────────────────────────────────
// Context Management
// ─────────────────────────────────────────────────────────────────────────────

/// Check if conductor context has been set up
pub fn context_exists(config: &Config) -> bool {
    let context_dir = Path::new(&config.conductor.context_dir);
    context_dir.exists()
        && context_dir.join("product.md").exists()
        && context_dir.join("tech_stack.md").exists()
        && context_dir.join("workflow.md").exists()
}

/// Create the conductor context directory structure
pub fn create_context_dirs(config: &Config) -> Result<()> {
    let context_dir = Path::new(&config.conductor.context_dir);
    fs::create_dir_all(context_dir)
        .with_context(|| format!("Failed to create context dir: {}", context_dir.display()))?;

    let tracks_dir = Path::new(&config.conductor.tracks_dir);
    fs::create_dir_all(tracks_dir)
        .with_context(|| format!("Failed to create tracks dir: {}", tracks_dir.display()))?;

    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Track Management
// ─────────────────────────────────────────────────────────────────────────────

/// A track represents a unit of work (typically one feature)
#[derive(Debug, Clone)]
pub struct Track {
    /// Slug name of the track (directory name)
    pub name: String,
    /// Path to the track directory
    pub path: std::path::PathBuf,
    /// Whether spec.md exists
    pub has_spec: bool,
    /// Whether plan.md exists
    pub has_plan: bool,
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
                        has_spec: path.join("spec.md").exists(),
                        has_plan: true,
                    }));
                }
            }
        }
    }

    Ok(None)
}

/// Create a new track directory for a feature
pub fn create_track(config: &Config, feature_name: &str) -> Result<Track> {
    let slug = slugify(feature_name);
    let tracks_dir = Path::new(&config.conductor.tracks_dir);
    let track_path = tracks_dir.join(&slug);

    fs::create_dir_all(&track_path)
        .with_context(|| format!("Failed to create track dir: {}", track_path.display()))?;

    Ok(Track {
        name: slug,
        path: track_path,
        has_spec: false,
        has_plan: false,
    })
}

/// Convert a feature name to a URL-safe slug
fn slugify(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

// ─────────────────────────────────────────────────────────────────────────────
// Plan Parsing
// ─────────────────────────────────────────────────────────────────────────────

/// A task from a plan.md file
#[derive(Debug, Clone)]
pub struct PlanTask {
    /// Line number in the file (1-indexed)
    pub line_number: usize,
    /// Task description (without checkbox)
    pub description: String,
    /// Whether the task is complete
    pub complete: bool,
    /// Indentation level (0 = top-level, 1 = subtask, etc.)
    pub level: usize,
}

/// Parse tasks from a plan.md file
pub fn parse_plan(plan_path: &Path) -> Result<Vec<PlanTask>> {
    let content = fs::read_to_string(plan_path)
        .with_context(|| format!("Failed to read plan: {}", plan_path.display()))?;

    let mut tasks = Vec::new();

    for (line_idx, line) in content.lines().enumerate() {
        let trimmed = line.trim_start();

        // Count indentation (spaces / 2 or tabs)
        let indent_chars = line.len() - trimmed.len();
        let level = indent_chars / 2;

        if let Some(rest) = trimmed.strip_prefix("- [ ]") {
            let description = rest.trim().to_string();
            tasks.push(PlanTask {
                line_number: line_idx + 1,
                description,
                complete: false,
                level,
            });
        } else if let Some(rest) = trimmed.strip_prefix("- [x]").or_else(|| trimmed.strip_prefix("- [X]")) {
            let description = rest.trim().to_string();
            tasks.push(PlanTask {
                line_number: line_idx + 1,
                description,
                complete: true,
                level,
            });
        }
    }

    Ok(tasks)
}

/// Get the next incomplete task from a plan
pub fn get_next_task(tasks: &[PlanTask]) -> Option<&PlanTask> {
    tasks.iter().find(|t| !t.complete)
}

/// Mark a task as complete in the plan file
pub fn mark_task_complete(plan_path: &Path, line_number: usize) -> Result<()> {
    let content = fs::read_to_string(plan_path)?;
    let mut lines: Vec<String> = content.lines().map(String::from).collect();

    if line_number > 0 && line_number <= lines.len() {
        let line = &mut lines[line_number - 1];
        if line.contains("- [ ]") {
            *line = line.replace("- [ ]", "- [x]");
        }
    }

    fs::write(plan_path, lines.join("\n") + "\n")?;
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("User Authentication"), "user-authentication");
        assert_eq!(slugify("API_Endpoint_v2"), "api-endpoint-v2");
        assert_eq!(slugify("  Multiple   Spaces  "), "multiple-spaces");
        assert_eq!(slugify("Special!@#$%^&*()Chars"), "special-chars");
    }

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
