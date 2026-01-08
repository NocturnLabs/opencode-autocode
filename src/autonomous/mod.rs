//! Autonomous agent runner
//!
//! Runs OpenCode in batch mode with automatic session continuation
//! until all features pass.

pub mod debug_logger;
mod decision;
mod display;
mod features;
mod git;
pub mod parallel;
pub mod runner;
pub mod security;
mod session;
mod settings;
mod stats;
pub mod supervisor;
pub mod templates;
pub mod verification;
mod verifier;
mod webhook;

use anyhow::Result;
use std::path::Path;
use std::time::Duration;

use crate::config::Config;

use features::FeatureProgress;
use settings::LoopSettings;

/// Run parallel workers using git worktrees
pub fn run_parallel(
    worker_count: usize,
    limit: Option<usize>,
    config_path: Option<&Path>,
    developer_mode: bool,
) -> Result<()> {
    let pid = std::process::id();
    let log_path = format!("opencode-parallel-{}.log", pid);

    let (config, settings) =
        init_session(developer_mode, config_path, limit, false, Some(&log_path))?;
    let logger = debug_logger::get();
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

        logger.separator();
        logger.info(&format!("Parallel Iteration {} starting", iteration));
        display::display_session_header(iteration);

        println!("📋 Selected {} features for parallel work:", pending.len());
        for f in &pending {
            println!("   • #{}: {}", f.id.unwrap_or(0), f.description);
        }

        let base_path = std::env::current_dir()?;
        let mut coordinator = parallel::Coordinator::new(worker_count, base_path.clone());

        // Create worktrees and spawn workers
        let mut handles = Vec::new();
        for feature in pending {
            let (worktree_path, branch_name) =
                parallel::create_worktree(&feature, &base_path, &config)?;
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

                parallel::WorkerResult {
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

/// Run the autonomous agent loop
pub fn run(
    limit: Option<usize>,
    config_path: Option<&Path>,
    developer_mode: bool,
    single_model: bool,
    enhancement_mode: bool,
    target_feature_id: Option<i64>,
) -> Result<()> {
    let pid = std::process::id();
    // Use a simpler log name for the main "vibe" command, but still PID-scoped if needed.
    // However, if we want to allow `tail -f opencode-debug.log`, maybe we should symlink it?
    // For now, let's use a unique name so we can distinguish instances.
    let log_path = format!("opencode-debug-{}.log", pid);

    let (config, settings) = init_session(
        developer_mode,
        config_path,
        limit,
        single_model,
        Some(&log_path),
    )?;
    let logger = debug_logger::get();

    // Register instance globally
    let instance_repo = crate::db::InstanceRepository::open()?;
    let instance_id = instance_repo.register(pid, "supervisor", Some(&log_path))?;
    logger.info(&format!("Process registered as instance #{}", instance_id));

    // Register Ctrl+C handler to create stop signal file AND update DB status
    // Repository handles its own connection, so we can clone/re-open safely
    ctrlc::set_handler(move || {
        std::fs::write(session::STOP_SIGNAL_FILE, "").ok();
        println!("\n→ Ctrl+C detected, stopping after current session...");

        // Try to verify instance stopped status
        if let Ok(repo) = crate::db::InstanceRepository::open() {
            let _ = repo.mark_stopped(instance_id);
        }
    })
    .ok();

    logger.separator();
    logger.separator();
    logger.info("OpenCode Supervisor starting");

    log_startup_info(logger, &settings, developer_mode, single_model);

    display::display_banner(
        &settings.model,
        settings.max_iterations,
        settings.delay_seconds,
        developer_mode,
    );

    let result =
        supervisor::run_supervisor_loop(&config, &settings, enhancement_mode, target_feature_id);

    // Final status display and cleanup
    log_final_status(&settings, developer_mode);
    let _ = instance_repo.mark_stopped(instance_id);

    result
}

fn log_final_status(settings: &settings::LoopSettings, developer_mode: bool) {
    let logger = debug_logger::get();
    let db_path = Path::new(&settings.database_file);

    // Get passing stats safely
    let (passing, total) = if db_path.exists() {
        FeatureProgress::load_from_db(db_path)
            .map(|p| (p.passing, p.total()))
            .unwrap_or((0, 0))
    } else {
        (0, 0)
    };

    logger.separator();
    logger.info(&format!(
        "Supervisor stopped. Final status: {}/{} tests passing",
        passing, total
    ));
    logger.separator();

    display::display_final_status(passing, total, developer_mode);
}

fn log_startup_info(
    logger: &debug_logger::DebugLogger,
    settings: &settings::LoopSettings,
    developer_mode: bool,
    single_model: bool,
) {
    logger.info(&format!("Developer mode: {}", developer_mode));
    logger.info(&format!(
        "Project directory: {}",
        std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_default()
    ));
    logger.info(&format!("Model: {}", settings.model));
    logger.info(&format!(
        "Dual-model: {}",
        if single_model {
            "disabled (--single-model)"
        } else {
            "enabled (reasoning + @coder)"
        }
    ));
    logger.info(&format!(
        "MCP: {} required tools configured",
        settings.mcp.required_tools.len()
    ));
    logger.info(&format!(
        "Max iterations: {}",
        if settings.max_iterations == usize::MAX {
            "unlimited".to_string()
        } else {
            settings.max_iterations.to_string()
        }
    ));
    logger.info(&format!(
        "Session timeout: {} minutes",
        settings.session_timeout
    ));
    logger.separator();
}

fn load_config(config_path: Option<&Path>) -> Result<Config> {
    match config_path {
        Some(path) => Config::load_from_file(path),
        None => Config::load(None),
    }
}

/// Initialize a session, loading config and setting up logging
fn init_session(
    developer_mode: bool,
    config_path: Option<&Path>,
    limit: Option<usize>,
    single_model: bool,
    log_path: Option<&str>,
) -> Result<(Config, LoopSettings)> {
    debug_logger::init(developer_mode, log_path);
    let config = load_config(config_path)?;
    let mut settings = settings::LoopSettings::from_config(&config, limit);
    settings.dual_model_enabled = !single_model;

    // Clear any lingering stop signal from a previous run
    session::clear_stop_signal();

    Ok((config, settings))
}
