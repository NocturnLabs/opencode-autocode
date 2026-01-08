use anyhow::Result;
use std::path::PathBuf;

use super::merge::rebase_and_merge;
use super::types::WorkerResult;
use super::worktree::remove_worktree;
use crate::autonomous::git;

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
