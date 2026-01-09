//! Autonomous agent runner
//!
//! Runs OpenCode in batch mode with automatic session continuation
//! until all features pass.

mod decision;
mod display;
mod features;
mod git;
pub mod parallel;
pub mod runner;
pub mod security;
mod session;
pub mod settings;
mod stats;
pub mod supervisor;
pub mod templates;
pub mod verification;
mod verifier;
mod webhook;

use anyhow::Result;
use std::path::Path;

use features::FeatureProgress;

// Re-export run_parallel from the parallel module
pub use parallel::run_parallel;

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

    let (config, settings) = settings::init_session(
        developer_mode,
        config_path,
        limit,
        single_model,
        Some(&log_path),
    )?;
    let logger = crate::common::logging::get();

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
    let logger = crate::common::logging::get();
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
    logger: &crate::common::logging::DebugLogger,
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
