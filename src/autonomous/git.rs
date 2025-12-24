//! Git operations for auto-commit on feature completion

use anyhow::{Context, Result};
use std::process::Command;

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
        anyhow::bail!("git add failed with exit code {}", status.code().unwrap_or(-1));
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
