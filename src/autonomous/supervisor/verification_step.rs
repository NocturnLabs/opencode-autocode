use anyhow::Result;
use std::path::Path;

use crate::autonomous::settings::LoopSettings;
use crate::autonomous::verifier::{
    handle_verification_failure, handle_verification_success, run_verification, VerificationResult,
};
use crate::config::Config;
use crate::db::features::Feature;

pub fn perform_verification(
    feature: &Feature,
    db_path: &Path,
    config: &Config,
    settings: &LoopSettings,
    iteration: usize,
    last_run_success: &mut bool,
) -> Result<bool> {
    println!("🔍 Supervisor: Verifying feature...");
    println!("   Feature: {}", feature.description);

    let verification_result = run_verification(feature, &config.security)?;
    let mut made_progress = false;

    match verification_result {
        VerificationResult::Passed => {
            *last_run_success = true;
            made_progress = true;
            handle_verification_success(feature, db_path, config, settings, iteration)?;
        }
        VerificationResult::Failed { error_message } => {
            *last_run_success = false;
            handle_verification_failure(feature, &error_message, db_path, settings)?;
        }
        VerificationResult::NoCommand => {
            println!("  ❌ No verification command (manual check required)");
            *last_run_success = false;
            let db = crate::db::Database::open(db_path)?;
            db.features().mark_failing_with_error(
                &feature.description,
                Some("No verification command produced by agent"),
            )?;
        }
        VerificationResult::SecurityBlocked { reason } => {
            println!("  🚫 Security: Command blocked");
            println!("     {}", reason);
            *last_run_success = false;
            let db = crate::db::Database::open(db_path)?;
            db.features().mark_failing_with_error(
                &feature.description,
                Some(&format!("Security blocked: {}", reason)),
            )?;
        }
    }

    Ok(made_progress)
}
