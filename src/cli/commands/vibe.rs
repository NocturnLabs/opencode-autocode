use anyhow::Result;
use num_cpus;

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
