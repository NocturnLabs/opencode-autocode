use std::path::PathBuf;

/// Result of a worker completing a feature
#[derive(Debug)]
pub struct WorkerResult {
    pub feature_id: i64,
    pub branch_name: String,
    pub worktree_path: PathBuf,
    pub success: bool,
}
