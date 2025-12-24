//! Webhook notifications for completed features

use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

use crate::config::Config;
use crate::regression::Feature;

/// Send webhook notification for a completed feature
pub fn notify_feature_complete(
    config: &Config,
    feature: &Feature,
    session_number: usize,
    current_passing: usize,
    total_features: usize,
) -> Result<()> {
    if !config.notifications.webhook_enabled {
        return Ok(());
    }

    let url = match &config.notifications.webhook_url {
        Some(u) => u,
        None => return Ok(()),
    };

    println!(
        "→ Sending webhook notification for: {}",
        feature.description
    );

    let payload = build_webhook_payload(feature, session_number, current_passing, total_features)?;
    send_webhook_request(url, &payload)?;

    Ok(())
}

fn build_webhook_payload(
    feature: &Feature,
    session_number: usize,
    current_passing: usize,
    total_features: usize,
) -> Result<String> {
    let project_name = std::env::current_dir()?
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown Project")
        .to_string();

    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let verification_steps = if feature.steps.is_empty() {
        "No steps defined".to_string()
    } else {
        feature
            .steps
            .iter()
            .take(5)
            .map(|s| format!("• {}", s))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let progress_percent = if total_features > 0 {
        (current_passing * 100) / total_features
    } else {
        0
    };

    // Prepare template data
    let mut data = std::collections::HashMap::new();
    data.insert("feature_name", feature.description.clone());
    data.insert("feature_category", capitalize_first(&feature.category));
    data.insert("project_name", project_name);
    data.insert("timestamp", timestamp);
    data.insert("session_number", session_number.to_string());
    data.insert("progress_current", current_passing.to_string());
    data.insert("progress_total", total_features.to_string());
    data.insert("progress_percent", progress_percent.to_string());
    data.insert("verification_steps", verification_steps);

    // Render template
    let handlebars = handlebars::Handlebars::new();
    let template_path = Path::new("templates/notifications/webhook.json");
    let template_content = if template_path.exists() {
        std::fs::read_to_string(template_path)?
    } else {
        r#"{ "content": "✅ Feature Completed: {{feature_name}} in {{project_name}} ({{progress_current}}/{{progress_total}})" }"#.to_string()
    };

    handlebars
        .render_template(&template_content, &data)
        .context("Failed to render webhook template")
}

fn send_webhook_request(url: &str, payload: &str) -> Result<()> {
    let status = Command::new("curl")
        .arg("-X")
        .arg("POST")
        .arg("-H")
        .arg("Content-Type: application/json")
        .arg("-d")
        .arg(payload)
        .arg(url)
        .arg("--silent")
        .arg("--output")
        .arg("/dev/null")
        .status()?;

    if !status.success() {
        println!(
            "⚠ Failed to send webhook notification (curl exit {})",
            status
        );
    }

    Ok(())
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}
