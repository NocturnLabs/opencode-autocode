use anyhow::Result;
use std::path::Path;
use std::time::Duration;

use crate::autonomous::{display, features, session};

use super::coordinator::Coordinator;
use super::types::WorkerResult;
use super::worktree::create_worktree;

/// Run parallel workers using git worktrees
pub fn run_parallel(
    worker_count: usize,
    limit: Option<usize>,
    config_path: Option<&Path>,
    developer_mode: bool,
) -> Result<()> {
    let pid = std::process::id();
    let log_path = format!("opencode-parallel-{}.log", pid);

    // We need to access init_session which is in autonomous/mod.rs (parent).
    // But since this is a separate module, we can access public functions of crate::autonomous
    // run_parallel was originally in mod.rs and had access to private init_session.
    // We should probably expose init_session or duplicate the simple init logic.
    // Ideally refactor init_session to be public in settings or session.
    // checking autonomous/mod.rs again... init_session is private there.

    let (config, settings) = crate::autonomous::settings::init_session(
        developer_mode,
        config_path,
        limit,
        false,
        Some(&log_path),
    )?;

    let logger = crate::common::logging::get();
    let db_path = Path::new(&settings.database_file);

    // Register instance globally
    let instance_repo = crate::db::InstanceRepository::open()?;
    let instance_id = instance_repo.register(pid, "coordinator", Some(&log_path))?;
    logger.info(&format!("Process registered as instance #{}", instance_id));

    let mut iteration = 0usize;

    loop {
        iteration += 1;

        if iteration > settings.max_iterations {
            logger.info("Reached max iterations");
            println!("\nReached max iterations ({})", settings.max_iterations);
            break;
        }

        if session::stop_signal_exists() {
            logger.info("Parallel Coordinator: Stop signal received.");
            break;
        }

        // Get pending features
        let pending = features::get_pending_features(db_path, worker_count)?;
        if pending.is_empty() {
            println!("✅ No pending features to work on");
            break;
        }

        // Calculate width for parallel mode (similar to banner logic)
        let title = "OpenCode Autonomous Agent";
        let title_w = 7 + crate::theming::visual_width(title);
        let model_w = 7 + 5 + crate::theming::visual_width(&settings.model); // "Model: " label is 7 chars + value
        let width = title_w.max(model_w).max(60); // Minimum 60 for parallel mode info

        logger.separator();
        logger.info(&format!("Parallel Iteration {} starting", iteration));
        display::display_session_header(iteration, width);

        println!("📋 Selected {} features for parallel work:", pending.len());
        for f in &pending {
            println!("   • #{}: {}", f.id.unwrap_or(0), f.description);
        }

        let base_path = std::env::current_dir()?;
        let mut coordinator = Coordinator::new(worker_count, base_path.clone());

        // Create worktrees and spawn workers
        let mut handles = Vec::new();
        for feature in pending {
            let (worktree_path, branch_name) = create_worktree(&feature, &base_path, &config)?;
            println!("🌳 Created worktree: {}", branch_name);

            let feature_id = feature.id.unwrap_or(0);
            let wt = worktree_path.clone();
            let bn = branch_name.clone();

            // Spawn worker thread
            let handle = std::thread::spawn(move || {
                let status = std::process::Command::new("opencode-forger")
                    .args([
                        "vibe",
                        "--limit",
                        "1",
                        "--feature-id",
                        &feature_id.to_string(),
                    ])
                    .current_dir(&wt)
                    .status();

                WorkerResult {
                    feature_id,
                    branch_name: bn,
                    worktree_path: wt,
                    success: status.map(|s| s.success()).unwrap_or(false),
                }
            });
            handles.push(handle);
        }

        // Wait for workers and queue results
        for handle in handles {
            match handle.join() {
                Ok(result) => {
                    println!(
                        "{}  Worker {} finished ({})",
                        if result.success { "✅" } else { "❌" },
                        result.feature_id,
                        if result.success { "success" } else { "failed" }
                    );
                    coordinator.queue_for_merge(result);
                }
                Err(e) => {
                    let msg = if let Some(s) = e.downcast_ref::<&str>() {
                        s
                    } else {
                        "unknown panic"
                    };
                    logger.error(&format!("Worker thread panicked: {}", msg));
                }
            }
        }

        // Process merge queue
        println!("\n📦 Processing merge queue...");
        let merged = coordinator.process_merge_queue()?;
        println!("✅ Merged {} features to main", merged);

        logger.info(&format!(
            "Parallel iteration complete: {} workers, {} merged",
            worker_count, merged
        ));

        // Delay between iterations
        if iteration < settings.max_iterations {
            std::thread::sleep(Duration::from_secs(settings.delay_seconds as u64));
        }
    }

    // Mark instance as stopped
    let _ = instance_repo.mark_stopped(instance_id);

    Ok(())
}
