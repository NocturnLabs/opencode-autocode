//! Git operations for auto-commit on feature completion
//!
//! Provides both commit operations and high-level git helpers for
//! parallel worktree coordination.

use anyhow::{Context, Result};
use std::process::Command;

// ─────────────────────────────────────────────────────────────────────────────
// Commit Operations
// ─────────────────────────────────────────────────────────────────────────────

/// Auto-commit a completed feature to git
pub fn commit_completed_feature(feature_description: &str, verbose: bool) -> Result<()> {
    stage_all_changes()?;
    create_feature_commit(feature_description, verbose)?;
    Ok(())
}

fn stage_all_changes() -> Result<()> {
    let status = Command::new("git")
        .args(["add", "."])
        .status()
        .context("Failed to run git add")?;

    if !status.success() {
        anyhow::bail!(
            "git add failed with exit code {}",
            status.code().unwrap_or(-1)
        );
    }

    Ok(())
}

fn create_feature_commit(feature_description: &str, verbose: bool) -> Result<()> {
    let commit_msg = format!("feat: {}", feature_description);

    let status = Command::new("git")
        .args(["commit", "-m", &commit_msg])
        .status()
        .context("Failed to run git commit")?;

    if status.success() {
        if verbose {
            println!("✓ Auto-committed: {}", commit_msg);
        }
    } else {
        // Exit code 1 often means "nothing to commit" which is OK
        if verbose {
            println!("→ Git commit skipped (nothing to commit)");
        }
    }

    Ok(())
}

/// Discard all uncommitted changes to reset the working directory
/// Used when verification fails to give the next attempt a clean slate
pub fn discard_changes(verbose: bool) -> Result<()> {
    // Reset tracked files
    let checkout = Command::new("git")
        .args(["checkout", "."])
        .status()
        .context("Failed to run git checkout")?;

    // Remove untracked files
    let clean = Command::new("git")
        .args(["clean", "-fd"])
        .status()
        .context("Failed to run git clean")?;

    if checkout.success() && clean.success() {
        if verbose {
            println!("✓ Discarded uncommitted changes");
        }
    } else if verbose {
        println!("→ Warning: Could not fully discard changes");
    }

    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// High-level Git Operations (for parallel worktree coordination)
// ─────────────────────────────────────────────────────────────────────────────

/// Checkout a branch, returning true if successful
pub fn checkout_branch(branch: &str) -> Result<bool> {
    let status = Command::new("git")
        .args(["checkout", branch])
        .status()
        .with_context(|| format!("Failed to checkout branch: {}", branch))?;

    Ok(status.success())
}

/// Stash current changes with a message, return true if anything was stashed
pub fn stash_push(message: &str) -> Result<bool> {
    let output = Command::new("git")
        .args(["stash", "push", "-m", message])
        .output()
        .context("Failed to run git stash push")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.contains("Saved working directory and index state"))
}

/// Pop the most recent stash
pub fn stash_pop() -> Result<bool> {
    let status = Command::new("git")
        .args(["stash", "pop"])
        .status()
        .context("Failed to run git stash pop")?;

    Ok(status.success())
}

/// Rebase a branch onto target (usually "main"), return true if successful
pub fn rebase(branch: &str, target: &str) -> Result<bool> {
    let status = Command::new("git")
        .args(["rebase", target, branch])
        .status()
        .with_context(|| format!("Failed to rebase {} onto {}", branch, target))?;

    if !status.success() {
        // Abort the failed rebase to restore clean state
        let _ = Command::new("git").args(["rebase", "--abort"]).status();
    }

    Ok(status.success())
}

/// Fast-forward merge a branch into current branch
pub fn merge_ff_only(branch: &str) -> Result<bool> {
    let status = Command::new("git")
        .args(["merge", "--ff-only", branch])
        .status()
        .with_context(|| format!("Failed to merge branch: {}", branch))?;

    Ok(status.success())
}

/// Delete a branch (force)
pub fn delete_branch_force(branch: &str) -> Result<()> {
    let _ = Command::new("git").args(["branch", "-D", branch]).output();
    Ok(())
}

/// Delete a branch (safe - only if merged)
pub fn delete_branch(branch: &str) -> Result<bool> {
    let status = Command::new("git")
        .args(["branch", "-d", branch])
        .status()
        .context("Failed to delete branch")?;

    Ok(status.success())
}
