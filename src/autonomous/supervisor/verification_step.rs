use anyhow::Result;
use std::path::Path;

use crate::autonomous::settings::LoopSettings;
use crate::autonomous::verifier::{
    handle_verification_failure, handle_verification_success, run_verification, VerificationResult,
};
use crate::config::Config;
use crate::db::features::Feature;

/// Outcome of verifying a feature.
pub struct VerificationOutcome {
    /// Whether verification progressed the loop.
    pub made_progress: bool,
    /// Optional error context from verification failure.
    pub error_context: Option<String>,
}

/// @param feature Feature under verification.
/// @param db_path Path to the feature database.
/// @param config Loaded configuration.
/// @param settings Loop settings for the supervisor.
/// @param iteration Current iteration number.
/// @param last_run_success Whether the last run succeeded.
/// @returns Verification outcome data.
pub fn perform_verification(
    feature: &Feature,
    db_path: &Path,
    config: &Config,
    settings: &LoopSettings,
    iteration: usize,
    last_run_success: &mut bool,
) -> Result<VerificationOutcome> {
    println!("🔍 Supervisor: Verifying feature...");
    println!("   Feature: {}", feature.description);

    let verification_result = run_verification(feature, &config.security)?;
    let mut made_progress = false;
    let mut error_context = None;

    match verification_result {
        VerificationResult::Passed => {
            *last_run_success = true;
            made_progress = true;
            handle_verification_success(feature, db_path, config, settings, iteration)?;
        }
        VerificationResult::Failed { error_message } => {
            *last_run_success = false;
            error_context = Some(error_message.clone());
            handle_verification_failure(feature, &error_message, db_path, settings)?;
        }
        VerificationResult::NoCommand => {
            if config.features.require_verification_command {
                println!("  ❌ No verification command (manual check required)");
                *last_run_success = false;
                error_context = Some("No verification command produced by agent".to_string());
                let db = crate::db::Database::open(db_path)?;
                db.features().mark_failing_with_error(
                    &feature.description,
                    Some("No verification command produced by agent"),
                )?;
            } else {
                println!("  ⚠️ No verification command; marking as manually verified");
                *last_run_success = true;
                made_progress = true;
                handle_verification_success(feature, db_path, config, settings, iteration)?;
            }
        }
        VerificationResult::SecurityBlocked { reason } => {
            println!("  🚫 Security: Command blocked");
            println!("     {}", reason);
            *last_run_success = false;
            error_context = Some(format!("Security blocked: {}", reason));
            let db = crate::db::Database::open(db_path)?;
            db.features().mark_failing_with_error(
                &feature.description,
                Some(&format!("Security blocked: {}", reason)),
            )?;
        }
    }

    Ok(VerificationOutcome {
        made_progress,
        error_context,
    })
}
