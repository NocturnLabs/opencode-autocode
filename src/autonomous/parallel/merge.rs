use crate::autonomous::git;
use anyhow::Result;

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
