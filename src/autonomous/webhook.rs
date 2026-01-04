//! Webhook notifications for completed features

use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

use crate::config::Config;
use crate::db::{features::Feature, Database, DEFAULT_DB_PATH};

/// Trait for sending webhook requests (allows mocking)
pub trait WebhookSender {
    fn send(&self, url: &str, payload: &str, method: &str) -> Result<()>;
    fn send_with_response(&self, url: &str, payload: &str, method: &str) -> Result<String>;
}

/// Default sender using curl
pub struct CurlSender;

impl WebhookSender for CurlSender {
    fn send(&self, url: &str, payload: &str, method: &str) -> Result<()> {
        let status = Command::new("curl")
            .arg("-X")
            .arg(method)
            .arg("-H")
            .arg("Content-Type: application/json")
            .arg("-d")
            .arg(payload)
            .arg(url)
            .arg("--silent")
            .arg("--output")
            .arg("/dev/null")
            .arg("--fail")
            .status()?;

        if !status.success() {
            anyhow::bail!("Request failed (curl exit {})", status);
        }
        Ok(())
    }

    fn send_with_response(&self, url: &str, payload: &str, method: &str) -> Result<String> {
        let output = Command::new("curl")
            .arg("-X")
            .arg(method)
            .arg("-H")
            .arg("Content-Type: application/json")
            .arg("-d")
            .arg(payload)
            .arg(url)
            .arg("--silent")
            .arg("--fail")
            .output()?;

        if !output.status.success() {
            anyhow::bail!("Request failed (curl exit {})", output.status);
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

/// Send webhook notification which updates a single dashboard message
pub fn notify_feature_complete(
    config: &Config,
    feature: &Feature,
    session_number: usize,
    current_passing: usize,
    total_features: usize,
) -> Result<()> {
    notify_with_sender(
        config,
        feature,
        session_number,
        current_passing,
        total_features,
        &CurlSender,
        DEFAULT_DB_PATH,
    )
}

/// Internal notification logic with injectable sender and DB path
fn notify_with_sender<S: WebhookSender>(
    config: &Config,
    feature: &Feature,
    session_number: usize,
    current_passing: usize,
    total_features: usize,
    sender: &S,
    db_path_str: &str,
) -> Result<()> {
    if !config.notifications.webhook_enabled {
        return Ok(());
    }

    let url = match &config.notifications.webhook_url {
        Some(u) => u,
        None => return Ok(()),
    };

    println!("→ Updating webhook dashboard for: {}", feature.description);

    // Try to load existing message ID
    let db_path = Path::new(db_path_str);
    let message_id = if db_path.exists() {
        match Database::open(db_path) {
            Ok(db) => db.meta().get("discord_message_id").unwrap_or(None),
            Err(_) => None,
        }
    } else {
        None
    };

    let payload = build_webhook_payload(feature, session_number, current_passing, total_features)?;

    match message_id {
        Some(id) => {
            // PATCH existing message
            let update_url = format!("{}/messages/{}", url, id);
            if sender.send(&update_url, &payload, "PATCH").is_err() {
                // If PATCH fails, create new
                create_new_message(url, &payload, sender, db_path_str)?;
            }
        }
        None => {
            // POST new message
            create_new_message(url, &payload, sender, db_path_str)?;
        }
    }

    Ok(())
}

fn create_new_message<S: WebhookSender>(
    url: &str,
    payload: &str,
    sender: &S,
    db_path_str: &str,
) -> Result<()> {
    // Discord requires ?wait=true to return the message object (and its ID)
    let mut final_url = url.to_string();
    if final_url.contains("discord.com") && !final_url.contains("wait=true") {
        let separator = if final_url.contains('?') { "&" } else { "?" };
        final_url.push_str(separator);
        final_url.push_str("wait=true");
    }

    let response = sender.send_with_response(&final_url, payload, "POST")?;

    if let Some(id) = extract_json_id(&response) {
        let db_path = Path::new(db_path_str);
        // We need to ensure DB exists in test context, but typically it should
        if let Ok(db) = Database::open(db_path) {
            let _ = db.meta().set("discord_message_id", &id);
        }
    }
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

    let progress_percent = if total_features > 0 {
        (current_passing * 100) / total_features
    } else {
        0
    };

    // Visual progress bar
    let bar_len = 20;
    // Clippy fix: progress_percent is already usize (u64 cast to usize elsewhere or implied) - checking context
    // Actually progress_percent comes from earlier calculation. Let's see.
    // Wait, the error said `progress_percent as usize` is unnecessary.
    // And `render_template` takes `&str` and we passed `&template_content` where `template_content` is `&str` (from include_str!).

    let filled = (progress_percent * bar_len) / 100;
    let empty = bar_len - filled;
    let progress_bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));

    // Prepare template data
    let mut data = std::collections::HashMap::new();
    data.insert("feature_name", feature.description.clone());
    data.insert("feature_category", capitalize_first(&feature.category));
    data.insert("project_name", project_name);
    data.insert("timestamp", timestamp);
    data.insert("session_number", session_number.to_string());
    data.insert("progress_current", current_passing.to_string());
    data.insert(
        "progress_remaining",
        (total_features - current_passing).to_string(),
    );
    data.insert("progress_total", total_features.to_string());
    data.insert("progress_percent", progress_percent.to_string());
    data.insert("progress_bar", progress_bar);

    // Render template
    let handlebars = handlebars::Handlebars::new();

    // Embed the template directly into the binary
    let template_content = include_str!("../../templates/notifications/webhook.json");
    // Fallback logic is no longer needed since inclusion is verified at compile time

    handlebars
        .render_template(template_content, &data) // Clippy fix: remove &
        .context("Failed to render webhook template")
}

fn extract_json_id(json: &str) -> Option<String> {
    // Robustly find "id": "value" even with varying whitespace
    let id_key = "\"id\"";
    if let Some(key_idx) = json.find(id_key) {
        let rest = &json[key_idx + id_key.len()..];
        // Find the colon and the opening quote for the value
        if let Some(colon_idx) = rest.find(':') {
            let val_part = &rest[colon_idx + 1..].trim_start();
            if let Some(stripped) = val_part.strip_prefix('"') {
                if let Some(end_idx) = stripped.find('"') {
                    return Some(stripped[..end_idx].to_string());
                }
            }
        }
    }
    None
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use tempfile::NamedTempFile;

    struct MockSender {
        last_method: Arc<Mutex<String>>,
        last_url: Arc<Mutex<String>>,
        response_id: String,
    }

    impl WebhookSender for MockSender {
        fn send(&self, url: &str, _payload: &str, method: &str) -> Result<()> {
            *self.last_method.lock().unwrap() = method.to_string();
            *self.last_url.lock().unwrap() = url.to_string();
            Ok(())
        }

        fn send_with_response(&self, url: &str, _payload: &str, method: &str) -> Result<String> {
            *self.last_method.lock().unwrap() = method.to_string();
            *self.last_url.lock().unwrap() = url.to_string();
            Ok(format!("{{ \"id\": \"{}\" }}", self.response_id))
        }
    }

    #[test]
    fn test_webhook_dashboard_flow() {
        // Setup DB
        let db_file = NamedTempFile::new().unwrap();
        let db_path_str = db_file.path().to_str().unwrap();
        let _ = Database::open(db_file.path()).unwrap();

        // Setup Config
        let mut config = Config::default();
        config.notifications.webhook_enabled = true;
        config.notifications.webhook_url =
            Some("http://discord.com/api/webhooks/123/token".to_string());

        // Setup Feature
        let feature = Feature {
            id: Some(1),
            category: "test".to_string(),
            description: "Test Loop".to_string(),
            passes: true,
            verification_command: None,
            steps: vec![],
            last_error: None,
        };

        // Setup Mock Sender
        let last_method = Arc::new(Mutex::new(String::new()));
        let last_url = Arc::new(Mutex::new(String::new()));
        let sender = MockSender {
            last_method: last_method.clone(),
            last_url: last_url.clone(),
            response_id: "999999".to_string(),
        };

        // 1. First Call (Should be POST)
        notify_with_sender(&config, &feature, 1, 1, 10, &sender, db_path_str).unwrap();
        assert_eq!(*last_method.lock().unwrap(), "POST");

        // Verify ID saved to DB
        let db = Database::open(db_file.path()).unwrap();
        let saved_id = db.meta().get("discord_message_id").unwrap();
        assert_eq!(saved_id, Some("999999".to_string()));

        // 2. Second Call (Should be PATCH)
        notify_with_sender(&config, &feature, 2, 2, 10, &sender, db_path_str).unwrap();
        assert_eq!(*last_method.lock().unwrap(), "PATCH");
        assert!(last_url.lock().unwrap().ends_with("/messages/999999"));
    }
}
