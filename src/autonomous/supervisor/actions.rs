use anyhow::Result;
use std::path::Path;

use crate::autonomous::features::{get_feature_by_id, get_first_pending_feature};
use crate::config::Config;
use crate::db::features::Feature;

use crate::autonomous::decision::SupervisorAction;
use crate::autonomous::settings::{LoopAction, LoopSettings};
use crate::autonomous::templates;
use crate::common::logging::DebugLogger;

pub struct ActionCommand {
    pub name: String,
    pub active_feature: Option<Feature>,
    pub should_break: bool,
    pub no_progress: bool,
}

pub fn prepare_command(
    action: SupervisorAction,
    enhancement_mode: bool,
    config: &Config,
    settings: &LoopSettings,
    iteration: &mut usize,
    logger: &DebugLogger,
    target_feature_id: Option<i64>,
) -> Result<ActionCommand> {
    let db_path = Path::new(&settings.database_file);

    match action {
        SupervisorAction::Complete => {
            // In normal mode, this is handled by early exit above
            // In enhancement_mode, route to enhancement phase
            if enhancement_mode {
                match templates::handle_enhancement_phase(db_path, config, settings, *iteration) {
                    Ok(LoopAction::Continue) => {
                        *iteration += 1;
                        Ok(ActionCommand {
                            name: "auto-enhance-active".to_string(),
                            active_feature: None,
                            should_break: false,
                            no_progress: false,
                        })
                    }
                    _ => {
                        logger.info("Supervisor: Enhancement phase exited.");
                        Ok(ActionCommand {
                            name: String::new(),
                            active_feature: None,
                            should_break: true,
                            no_progress: false,
                        })
                    }
                }
            } else {
                logger.info("Supervisor: All features complete.");
                Ok(ActionCommand {
                    name: String::new(),
                    active_feature: None,
                    should_break: true,
                    no_progress: false,
                })
            }
        }
        SupervisorAction::EnhanceReady => {
            // Same as Complete for now, enhancement mode handles this
            match templates::handle_enhancement_phase(db_path, config, settings, *iteration) {
                Ok(LoopAction::Continue) => {
                    *iteration += 1;
                    Ok(ActionCommand {
                        name: "auto-enhance-active".to_string(),
                        active_feature: None,
                        should_break: false,
                        no_progress: false,
                    })
                }
                _ => {
                    logger.info("Supervisor: Enhancement phase exited.");
                    Ok(ActionCommand {
                        name: String::new(),
                        active_feature: None,
                        should_break: true,
                        no_progress: false,
                    })
                }
            }
        }
        SupervisorAction::Command(cmd) => {
            logger.info(&format!("Supervisor: Selected command '{}'", cmd));

            // For auto-continue, inject feature context (supervisor controls what LLM works on)
            if cmd == "auto-continue" {
                let feature_opt = if let Some(id) = target_feature_id {
                    get_feature_by_id(db_path, id)?
                } else {
                    get_first_pending_feature(db_path)?
                };

                if let Some(feature) = feature_opt {
                    templates::generate_continue_template(&feature, config)?;
                    println!(
                        "📋 Feature #{}: {}",
                        feature.id.unwrap_or(0),
                        feature.description
                    );
                    Ok(ActionCommand {
                        name: "auto-continue-active".to_string(),
                        active_feature: Some(feature),
                        should_break: false,
                        no_progress: false,
                    })
                } else {
                    // No pending features - continue without feature context (fallback template)
                    logger.warning("No pending feature found for auto-continue");
                    println!("⚠️ No pending feature found, running fallback template");
                    Ok(ActionCommand {
                        name: "auto-continue".to_string(),
                        active_feature: None,
                        should_break: false,
                        no_progress: false,
                    })
                }
            } else {
                Ok(ActionCommand {
                    name: cmd.to_string(),
                    active_feature: None,
                    should_break: false,
                    no_progress: false,
                })
            }
        }
        SupervisorAction::Fix { feature, error } => {
            logger.info(&format!(
                "Supervisor: REGRESSION DETECTED in '{}'",
                feature.description
            ));
            println!("🚨 REGRESSION DETECTED: {}", feature.description);
            println!("→ Switching to auto-fix mode...");

            // Generate dynamic auto-fix template
            templates::generate_fix_template(&feature, &error, db_path)?;
            Ok(ActionCommand {
                name: "auto-fix-active".to_string(),
                active_feature: Some(feature),
                should_break: false,
                no_progress: false,
            })
        }
    }
}
