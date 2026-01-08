//! Parallel feature development using git worktrees
//!
//! Enables multiple workers to implement features simultaneously,
//! each in their own worktree with a dedicated branch.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::autonomous::git;
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
pub fn create_worktree(
    feature: &Feature,
    base_path: &Path,
    config: &crate::config::Config,
) -> Result<(PathBuf, String)> {
    // Check for git index lock to avoid hanging/contention
    let lock_file = base_path.join(".git/index.lock");
    if lock_file.exists() {
        // Wait briefly to see if it clears (transient lock)
        std::thread::sleep(std::time::Duration::from_millis(1000));
        if lock_file.exists() {
            anyhow::bail!("Git index is locked (.git/index.lock exists). Please ensure no other git operations are running or remove the stale lock.");
        }
    }

    let feature_id = feature.id.unwrap_or(0);
    let slug = slugify(&feature.description);
    let branch_name = format!("feature/{}-{}", feature_id, slug);
    let worktree_path = base_path.join(&branch_name);

    // Clean up potential leftovers from previous runs
    if worktree_path.exists() {
        // Try git remove first
        if let Some(path_str) = worktree_path.to_str() {
            let _ = Command::new("git")
                .args(["worktree", "remove", "--force", path_str])
                .status();
        }

        // If directory still exists (e.g. untracked files and git failed), force delete
        if worktree_path.exists() {
            std::fs::remove_dir_all(&worktree_path).ok();
            // Prune to update git's internal state
            let _ = Command::new("git").args(["worktree", "prune"]).status();
        }
    }

    // Force delete the branch if it exists (fresh start for the feature execution)
    // We prioritize consistency over "resuming" potentially broken branch state.
    // However, we cannot delete a branch if we are currently on it in the main repo.
    let current_branch_output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output();
    if let Ok(output) = current_branch_output {
        let current = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if current == branch_name {
            // We are on the branch we want to delete, must move off it first
            git::checkout_branch("main")?;
        }
    }

    git::delete_branch_force(&branch_name)?;

    // Create the worktree with a new branch
    let worktree_str = worktree_path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Worktree path contains invalid UTF-8"))?;

    let status = Command::new("git")
        .args(["worktree", "add", worktree_str, "-b", &branch_name])
        .status()
        .context("Failed to create worktree")?;

    if !status.success() {
        anyhow::bail!("git worktree add failed for feature {}", feature_id);
    }

    // Ensure .forger directory exists in worktree before symlinking database files
    let worktree_forger_dir = worktree_path.join(".forger");
    if !worktree_forger_dir.exists() {
        std::fs::create_dir_all(&worktree_forger_dir)
            .with_context(|| "Failed to create .forger dir in worktree".to_string())?;
    }

    // Symlink the database file using the configured path (not hardcoded .forger/)
    let db_path = Path::new(&config.paths.database_file);
    let db_name = db_path
        .file_name()
        .unwrap_or_else(|| std::ffi::OsStr::new("progress.db"))
        .to_string_lossy();
    let db_parent = db_path.parent().unwrap_or(Path::new(".forger"));

    // Create parent directory in worktree for the database
    let worktree_db_parent = worktree_path.join(db_parent);
    if !worktree_db_parent.exists() {
        std::fs::create_dir_all(&worktree_db_parent).with_context(|| {
            format!(
                "Failed to create db parent dir in worktree: {}",
                worktree_db_parent.display()
            )
        })?;
    }

    // Symlink the database and WAL files to share progress between main and worktree.
    // CONCURRENCY NOTE: This creates a single point of contention as all workers
    // share the same SQLite database (with WAL mode enabled for better concurrency).
    // While WAL mode supports multiple readers and a single writer with busy timeouts,
    // very high parallelism (e.g., >4 workers) may experience lock contention.
    // Future hardening could consider per-worker DBs with merge, or a coordinator model.
    let db_files = [
        db_name.to_string(),
        format!("{}-shm", db_name),
        format!("{}-wal", db_name),
    ];
    for filename in &db_files {
        let main_file = std::env::current_dir()?.join(db_parent).join(filename);
        let worktree_file = worktree_path.join(db_parent).join(filename);

        if main_file.exists() && !worktree_file.exists() {
            #[cfg(unix)]
            std::os::unix::fs::symlink(&main_file, &worktree_file)
                .with_context(|| format!("Failed to symlink {}", filename))?;

            #[cfg(windows)]
            std::os::windows::fs::symlink_file(&main_file, &worktree_file)
                .with_context(|| format!("Failed to symlink {}", filename))?;
        }
    }

    // Also symlink forger.toml if it exists
    let main_config = std::env::current_dir()?.join("forger.toml");
    let worktree_config = worktree_path.join("forger.toml");
    if main_config.exists() && !worktree_config.exists() {
        #[cfg(unix)]
        std::os::unix::fs::symlink(&main_config, &worktree_config).ok();
        #[cfg(windows)]
        std::os::windows::fs::symlink_file(&main_config, &worktree_config).ok();
    }

    // Symlink .conductor context directory (shared planning context)
    let main_conductor = base_path.join(".conductor");
    let worktree_conductor = worktree_path.join(".conductor");
    if main_conductor.exists() && !worktree_conductor.exists() {
        #[cfg(unix)]
        std::os::unix::fs::symlink(&main_conductor, &worktree_conductor)
            .with_context(|| "Failed to symlink .conductor directory")?;
        #[cfg(windows)]
        std::os::windows::fs::symlink_dir(&main_conductor, &worktree_conductor)
            .with_context(|| "Failed to symlink .conductor directory")?;
    }

    // Symlink tracks directory (per-feature specs and plans)
    let main_tracks = base_path.join("tracks");
    let worktree_tracks = worktree_path.join("tracks");
    if main_tracks.exists() && !worktree_tracks.exists() {
        #[cfg(unix)]
        std::os::unix::fs::symlink(&main_tracks, &worktree_tracks)
            .with_context(|| "Failed to symlink tracks directory")?;
        #[cfg(windows)]
        std::os::windows::fs::symlink_dir(&main_tracks, &worktree_tracks)
            .with_context(|| "Failed to symlink tracks directory")?;
    }

    Ok((worktree_path, branch_name))
}

/// Remove a worktree after completion
pub fn remove_worktree(worktree_path: &Path, _branch_name: &str) -> Result<()> {
    // Force remove worktree (workers might have left untracked files)
    if let Some(path_str) = worktree_path.to_str() {
        let _ = Command::new("git")
            .args(["worktree", "remove", "--force", path_str])
            .status();
    }
    Ok(())
}

/// Rebase a branch onto main and fast-forward merge
pub fn rebase_and_merge(branch_name: &str) -> Result<bool> {
    // 1. Stash any changes in main
    let stashed = git::stash_push("Auto-stash before parallel merge")?;

    if !git::checkout_branch("main")? {
        // Restore stash before returning to avoid leaving stale stash entries
        if stashed {
            git::stash_pop().ok();
        }
        return Ok(false);
    }

    // 2. Rebase the feature branch onto main (this checks it out in the main repo)
    if !git::rebase(branch_name, "main")? {
        // ALWAYS return to main
        git::checkout_branch("main")?;
        // Restore stash before returning to avoid leaving stale stash entries
        if stashed {
            git::stash_pop().ok();
        }
        return Ok(false);
    }

    // 3. Checkout main again (rebase leaves you on the feature branch)
    git::checkout_branch("main")?;

    // 4. Fast-forward merge
    let success = git::merge_ff_only(branch_name)?;

    // 5. Pop stash if we stashed anything
    if stashed {
        git::stash_pop().ok();
    }

    Ok(success)
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
pub struct Coordinator {
    merge_queue: Vec<WorkerResult>,
}

impl Coordinator {
    pub fn new(_worker_count: usize, _base_path: PathBuf) -> Self {
        Self {
            merge_queue: Vec::new(),
        }
    }

    /// Queue a completed worker result for merging
    pub fn queue_for_merge(&mut self, result: WorkerResult) {
        // Queue EVERYTHING so we can clean up properly
        self.merge_queue.push(result);
    }

    /// Process the merge queue - clean up all, merge successful ones
    pub fn process_merge_queue(&mut self) -> Result<usize> {
        let mut merged_count = 0;

        for result in self.merge_queue.drain(..) {
            println!("📦 Processing result for: {}", result.branch_name);

            // ALWAYS remove worktree first
            println!("  → Removing worktree...");
            remove_worktree(&result.worktree_path, &result.branch_name)?;

            if result.success {
                println!("  → Merging feature...");
                if rebase_and_merge(&result.branch_name)? {
                    println!("  ✅ Merged successfully");
                    // Delete the merged branch
                    git::delete_branch(&result.branch_name).ok();
                    merged_count += 1;
                } else {
                    println!("  ⚠️ Rebase failed, branch left for manual review");
                }
            } else {
                println!("  ❌ Worker failed, skipping merge (branch preserved for debugging)");
                // We do NOT delete the branch here, so user can debug why it failed.
                // But we DID remove the worktree so we don't hog the filesystem.
            }
        }

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
