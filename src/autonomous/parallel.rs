//! Parallel feature development using git worktrees
//!
//! Enables multiple workers to implement features simultaneously,
//! each in their own worktree with a dedicated branch.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::db::features::Feature;

/// Result of a worker completing a feature
#[derive(Debug)]
pub struct WorkerResult {
    pub feature_id: i64,
    pub branch_name: String,
    pub worktree_path: PathBuf,
    pub success: bool,
}

/// Create a worktree for a feature, with shared config symlinked
pub fn create_worktree(feature: &Feature, base_path: &Path) -> Result<(PathBuf, String)> {
    let feature_id = feature.id.unwrap_or(0);
    let slug = slugify(&feature.description);
    let branch_name = format!("feature/{}-{}", feature_id, slug);
    let worktree_path = base_path.join(&branch_name);

    // Create the worktree with a new branch
    let status = Command::new("git")
        .args([
            "worktree",
            "add",
            worktree_path.to_str().unwrap(),
            "-b",
            &branch_name,
        ])
        .status()
        .context("Failed to create worktree")?;

    if !status.success() {
        anyhow::bail!("git worktree add failed for feature {}", feature_id);
    }

    // Symlink the database file specifically (the .autocode directory already exists
    // from tracked files, but progress.db is gitignored so needs to be symlinked)
    let main_db = std::env::current_dir()?.join(".autocode/progress.db");
    let worktree_db = worktree_path.join(".autocode/progress.db");
    
    if main_db.exists() && !worktree_db.exists() {
        #[cfg(unix)]
        std::os::unix::fs::symlink(&main_db, &worktree_db)
            .context("Failed to symlink progress.db")?;
        
        #[cfg(windows)]
        std::os::windows::fs::symlink_file(&main_db, &worktree_db)
            .context("Failed to symlink progress.db")?;
    }

    // Also symlink autocode.toml if it exists
    let main_config = std::env::current_dir()?.join("autocode.toml");
    let worktree_config = worktree_path.join("autocode.toml");
    if main_config.exists() && !worktree_config.exists() {
        #[cfg(unix)]
        std::os::unix::fs::symlink(&main_config, &worktree_config).ok();
        #[cfg(windows)]
        std::os::windows::fs::symlink_file(&main_config, &worktree_config).ok();
    }

    Ok((worktree_path, branch_name))
}

/// Remove a worktree after completion
pub fn remove_worktree(worktree_path: &Path, branch_name: &str) -> Result<()> {
    // Remove worktree
    let _ = Command::new("git")
        .args(["worktree", "remove", worktree_path.to_str().unwrap()])
        .status();

    // Delete the branch (it's been merged)
    let _ = Command::new("git")
        .args(["branch", "-d", branch_name])
        .status();

    Ok(())
}

/// Rebase a branch onto main and fast-forward merge
pub fn rebase_and_merge(branch_name: &str) -> Result<bool> {
    // Checkout main
    let status = Command::new("git")
        .args(["checkout", "main"])
        .status()
        .context("Failed to checkout main")?;

    if !status.success() {
        return Ok(false);
    }

    // Rebase the feature branch onto main
    let status = Command::new("git")
        .args(["rebase", "main", branch_name])
        .status()
        .context("Failed to rebase")?;

    if !status.success() {
        // Abort the rebase if it failed
        let _ = Command::new("git").args(["rebase", "--abort"]).status();
        return Ok(false);
    }

    // Fast-forward merge
    let status = Command::new("git")
        .args(["merge", "--ff-only", branch_name])
        .status()
        .context("Failed to merge")?;

    Ok(status.success())
}

/// Convert a description to a URL-safe slug
fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .take(5) // Limit length
        .collect::<Vec<_>>()
        .join("-")
}

/// Coordinator for parallel workers
#[allow(dead_code)]
pub struct Coordinator {
    worker_count: usize,
    base_path: PathBuf,
    merge_queue: Vec<WorkerResult>,
}

impl Coordinator {
    pub fn new(worker_count: usize, base_path: PathBuf) -> Self {
        Self {
            worker_count,
            base_path,
            merge_queue: Vec::new(),
        }
    }

    /// Queue a completed worker result for merging
    pub fn queue_for_merge(&mut self, result: WorkerResult) {
        if result.success {
            self.merge_queue.push(result);
        }
    }

    /// Process the merge queue - rebase and merge each branch sequentially
    pub fn process_merge_queue(&mut self) -> Result<usize> {
        let mut merged_count = 0;
        let mut retry_queue = Vec::new();

        for result in self.merge_queue.drain(..) {
            println!("📦 Merging: {}", result.branch_name);

            if rebase_and_merge(&result.branch_name)? {
                println!("  ✅ Merged successfully");
                remove_worktree(&result.worktree_path, &result.branch_name)?;
                merged_count += 1;
            } else {
                println!("  ⚠️ Rebase failed, re-queuing");
                retry_queue.push(result);
            }
        }

        // Re-add failed ones for another attempt
        self.merge_queue = retry_queue;
        Ok(merged_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("User authentication"), "user-authentication");
        assert_eq!(slugify("API: Login endpoint"), "api-login-endpoint");
        assert_eq!(
            slugify("very long feature description that goes on and on"),
            "very-long-feature-description-that"
        );
    }
}
