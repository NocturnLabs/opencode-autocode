use anyhow::{Context, Result};
use std::collections::VecDeque;
use std::path::Path;
use std::sync::mpsc;
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
    let log_name = format!("opencode-parallel-{}.log", pid);

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
        Some(&log_name),
    )?;

    let logger = crate::common::logging::get();
    let db_path = Path::new(&settings.database_file);

    // Register instance globally
    let instance_repo = crate::db::InstanceRepository::open()?;
    let instance_id = instance_repo.register(pid, "coordinator", settings.log_path.as_deref())?;
    logger.info(&format!("Process registered as instance #{}", instance_id));

    let mut iteration = 0usize;

    loop {
        iteration += 1;

        if iteration > settings.max_iterations {
            if settings.enforce_max_iterations {
                logger.info("Reached max iterations; stopping as requested");
                println!("\nReached max iterations ({})", settings.max_iterations);
                break;
            }

            logger.info("Reached max iterations; continuing until user stop");
            println!(
                "\nReached max iterations ({}), continuing until user stop",
                settings.max_iterations
            );
        }

        if session::stop_signal_exists() {
            logger.info("Parallel Coordinator: Stop signal received.");
            break;
        }

        // Get pending features
        let pending = features::get_pending_features(db_path, usize::MAX)?;
        if pending.is_empty() {
            println!("✅ No pending features to work on");
            break;
        }

        // Calculate width for parallel mode (similar to banner logic)
        let title = "OpenCode Autonomous Agent";
        let title_w = 7 + crate::theming::visual_width(title);
        let model_w = 7 + 5 + crate::theming::visual_width(&settings.coding_model); // "Model: " label is 7 chars + value
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
        let mut pending_queue: VecDeque<_> = pending.into();
        let (tx, rx) = mpsc::channel::<WorkerResult>();
        let mut active_workers = 0usize;
        let mut stop_requested = false;

        let spawn_worker = |feature: crate::db::features::Feature| -> Result<()> {
            let (worktree_path, branch_name) = create_worktree(&feature, &base_path, &config)?;
            println!("🌳 Created worktree: {}", branch_name);

            let feature_id = feature.id.unwrap_or(0);
            let wt = worktree_path.clone();
            let bn = branch_name.clone();
            let tx = tx.clone();

            std::thread::spawn(move || {
                let success = match std::panic::catch_unwind(|| {
                    std::process::Command::new("opencode-forger")
                        .args([
                            "vibe",
                            "--limit",
                            "1",
                            "--feature-id",
                            &feature_id.to_string(),
                        ])
                        .current_dir(&wt)
                        .status()
                        .map(|s| s.success())
                        .unwrap_or(false)
                }) {
                    Ok(success) => success,
                    Err(_) => {
                        eprintln!("Worker {} panicked", feature_id);
                        false
                    }
                };

                let _ = tx.send(WorkerResult {
                    feature_id,
                    branch_name: bn,
                    worktree_path: wt,
                    success,
                });
            });

            Ok(())
        };

        while active_workers < worker_count {
            if let Some(feature) = pending_queue.pop_front() {
                spawn_worker(feature)?;
                active_workers += 1;
            } else {
                break;
            }
        }

        while active_workers > 0 {
            if !stop_requested && session::stop_signal_exists() {
                stop_requested = true;
                logger.info("Parallel Coordinator: Stop signal received; waiting for workers.");
            }

            let result = rx.recv().context("Worker result channel closed")?;
            active_workers = active_workers.saturating_sub(1);

            println!(
                "{}  Worker {} finished ({})",
                if result.success { "✅" } else { "❌" },
                result.feature_id,
                if result.success { "success" } else { "failed" }
            );
            coordinator.queue_for_merge(result);

            if !stop_requested {
                if let Some(feature) = pending_queue.pop_front() {
                    spawn_worker(feature)?;
                    active_workers += 1;
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
