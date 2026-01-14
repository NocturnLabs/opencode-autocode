use anyhow::Result;
use num_cpus;

/// Handles the `vibe` subcommand for starting the autonomous coding loop.
///
/// This function determines whether to run in parallel mode (using git worktrees)
/// or sequential mode based on the provided `parallel` argument.
///
/// # Arguments
///
/// * `limit` - Maximum number of iterations (None for unlimited).
/// * `config_file` - Optional path to a custom config file.
/// * `developer` - Whether to enable developer mode for comprehensive debug logging.
/// * `single_model` - Whether to use a single model for all tasks (disables dual-model split).
/// * `parallel` - Number of workers for parallel execution (0 = auto-detect from CPU cores).
/// * `feature_id` - Optional specific feature ID to target (used by parallel workers).
///
/// # Returns
///
/// Result indicating success or containing an error from the autonomous runner.
pub fn handle_vibe(
    limit: Option<usize>,
    config_file: Option<&std::path::Path>,
    developer: bool,
    single_model: bool,
    parallel: Option<usize>,
    feature_id: Option<i64>,
) -> Result<()> {
    if let Some(worker_count) = parallel {
        // Parallel mode using worktrees
        let count = if worker_count == 0 {
            num_cpus::get() / 2 // Auto-detect: half of CPU cores
        } else {
            worker_count
        };
        println!("🔀 Starting parallel mode with {} workers", count);
        crate::autonomous::run_parallel(count, limit, config_file, developer)
    } else {
        // Standard sequential mode
        crate::autonomous::run(
            limit,
            config_file,
            developer,
            single_model,
            false,
            feature_id,
        )
    }
}
