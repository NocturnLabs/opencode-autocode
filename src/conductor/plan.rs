//! Plan file parsing and management
//!
//! Handles parsing of plan.md files and task manipulation.

use anyhow::Result;
use std::path::Path;

use crate::utils::{read_file, write_file};

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
    #[allow(dead_code)]
    pub level: usize,
}

/// Parse tasks from a plan.md file
pub fn parse_plan(plan_path: &Path) -> Result<Vec<PlanTask>> {
    let content = read_file(plan_path)?;

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
        } else if let Some(rest) = trimmed
            .strip_prefix("- [x]")
            .or_else(|| trimmed.strip_prefix("- [X]"))
        {
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
    let content = read_file(plan_path)?;
    let mut lines: Vec<String> = content.lines().map(String::from).collect();

    if line_number > 0 && line_number <= lines.len() {
        let line = &mut lines[line_number - 1];
        if line.contains("- [ ]") {
            *line = line.replace("- [ ]", "- [x]");
        }
    }

    write_file(plan_path, &(lines.join("\n") + "\n"))?;
    Ok(())
}
