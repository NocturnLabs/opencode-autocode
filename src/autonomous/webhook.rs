//! Webhook notifications for completed features

use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

use crate::config::Config;
use crate::db::{features::Feature, Database};

// ─────────────────────────────────────────────────────────────────────────────
// Failure Notification Types
// ─────────────────────────────────────────────────────────────────────────────

/// Reason for autonomous loop failure
#[derive(Debug, Clone)]
pub enum FailureReason {
    /// Reached maximum iteration limit
    MaxIterations { iterations: usize },
    /// No progress for consecutive iterations
    NoProgress { count: u32, limit: u32 },
    /// Fatal error during execution
    FatalError { message: String },
}

impl FailureReason {
    fn title(&self) -> &'static str {
        match self {
            FailureReason::MaxIterations { .. } => "🛑 Max Iterations Reached",
            FailureReason::NoProgress { .. } => "⚠️ No Progress Detected",
            FailureReason::FatalError { .. } => "❌ Fatal Error",
        }
    }

    fn description(&self) -> String {
        match self {
            FailureReason::MaxIterations { iterations } => {
                format!(
                    "The autonomous loop reached its maximum iteration limit of **{}**.",
                    iterations
                )
            }
            FailureReason::NoProgress { count, limit } => {
                format!(
                    "No progress was made for **{}** consecutive iterations (limit: {}).",
                    count, limit
                )
            }
            FailureReason::FatalError { message } => {
                format!("A fatal error occurred: {}", message)
            }
        }
    }

    fn color(&self) -> u32 {
        match self {
            FailureReason::MaxIterations { .. } => 15158332, // Red
            FailureReason::NoProgress { .. } => 16776960,    // Yellow
            FailureReason::FatalError { .. } => 10038562,    // Dark Red
        }
    }
}

/// Trait for sending webhook requests (allows mocking)
pub trait WebhookSender {
    fn send(&self, url: &str, payload: &str, method: &str) -> Result<()>;
    fn send_with_response(&self, url: &str, payload: &str, method: &str) -> Result<String>;
}

/// Default sender using curl
pub struct CurlSender;

impl WebhookSender for CurlSender {
    fn send(&self, url: &str, payload: &str, method: &str) -> Result<()> {
        use std::io::Write;
        use std::process::Stdio;

        // SECURITY: Pass payload via stdin (@-) to avoid exposing it in process lists
        let mut child = Command::new("curl")
            .arg("-X")
            .arg(method)
            .arg("-H")
            .arg("Content-Type: application/json")
            .arg("-d")
            .arg("@-") // Read payload from stdin
            .arg(url)
            .arg("--silent")
            .arg("--output")
            .arg("/dev/null")
            .arg("--fail")
            .stdin(Stdio::piped())
            .spawn()?;

        if let Some(ref mut stdin) = child.stdin {
            stdin.write_all(payload.as_bytes())?;
        }

        let status = child.wait()?;
        if !status.success() {
            anyhow::bail!("Request failed (curl exit {})", status);
        }
        Ok(())
    }

    fn send_with_response(&self, url: &str, payload: &str, method: &str) -> Result<String> {
        use std::io::Write;
        use std::process::Stdio;

        // SECURITY: Pass payload via stdin (@-) to avoid exposing it in process lists
        let mut child = Command::new("curl")
            .arg("-X")
            .arg(method)
            .arg("-H")
            .arg("Content-Type: application/json")
            .arg("-d")
            .arg("@-") // Read payload from stdin
            .arg(url)
            .arg("--silent")
            .arg("--fail")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        if let Some(ref mut stdin) = child.stdin {
            stdin.write_all(payload.as_bytes())?;
        }

        let output = child.wait_with_output()?;
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
        &config.paths.database_file,
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// Failure Notifications (Discord Bot API with Button)
// ─────────────────────────────────────────────────────────────────────────────

/// Send failure notification via Discord Bot API with restart button
pub fn notify_failure(config: &Config, reason: FailureReason) -> Result<()> {
    notify_failure_with_sender(config, reason, &CurlSender)
}

/// Internal failure notification logic with injectable sender
fn notify_failure_with_sender<S: WebhookSender>(
    config: &Config,
    reason: FailureReason,
    sender: &S,
) -> Result<()> {
    // Check if notifications are enabled
    if !config.notifications.webhook_enabled {
        return Ok(());
    }

    // Require both bot_token and channel_id for failure notifications
    let bot_token = match &config.notifications.bot_token {
        Some(t) if !t.is_empty() => t,
        _ => {
            println!("⚠️ Failure notification skipped: no bot_token configured");
            return Ok(());
        }
    };

    let channel_id = match &config.notifications.channel_id {
        Some(c) if !c.is_empty() => c,
        _ => {
            println!("⚠️ Failure notification skipped: no channel_id configured");
            return Ok(());
        }
    };

    println!("→ Sending failure notification: {}", reason.title());

    let payload = build_failure_payload(&reason)?;
    let url = format!(
        "https://discord.com/api/v10/channels/{}/messages",
        channel_id
    );

    // Use BotApiSender wrapper to add authorization header
    send_with_bot_auth(sender, &url, &payload, bot_token)?;

    Ok(())
}

/// Send request with Discord Bot authorization header
fn send_with_bot_auth<S: WebhookSender>(
    _sender: &S,
    url: &str,
    payload: &str,
    bot_token: &str,
) -> Result<()> {
    use std::io::Write;
    use std::process::Stdio;

    // SECURITY: Pass payload via stdin, token via header arg
    let mut child = Command::new("curl")
        .arg("-X")
        .arg("POST")
        .arg("-H")
        .arg("Content-Type: application/json")
        .arg("-H")
        .arg(format!("Authorization: Bot {}", bot_token))
        .arg("-d")
        .arg("@-")
        .arg(url)
        .arg("--silent")
        .arg("--fail")
        .stdin(Stdio::piped())
        .spawn()?;

    if let Some(ref mut stdin) = child.stdin {
        stdin.write_all(payload.as_bytes())?;
    }

    let status = child.wait()?;
    if !status.success() {
        anyhow::bail!("Discord Bot API request failed (curl exit {})", status);
    }
    Ok(())
}

/// Build failure notification payload with embed and restart button
fn build_failure_payload(reason: &FailureReason) -> Result<String> {
    let project_name = std::env::current_dir()?
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown Project")
        .to_string();

    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    // Build JSON payload with embed and button component
    let payload = serde_json::json!({
        "embeds": [{
            "title": reason.title(),
            "description": reason.description(),
            "color": reason.color(),
            "fields": [
                {
                    "name": "📁 Project",
                    "value": project_name,
                    "inline": true
                },
                {
                    "name": "🕐 Time",
                    "value": timestamp,
                    "inline": true
                }
            ],
            "footer": {
                "text": "OpenCode Forger Autonomous Loop"
            }
        }],
        "components": [{
            "type": 1,  // Action Row
            "components": [{
                "type": 2,       // Button
                "style": 3,      // Success (green)
                "label": "🔄 Restart Process",
                "custom_id": "restart_autonomous_loop"
            }]
        }]
    });

    Ok(payload.to_string())
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

/// Escape a string for safe inclusion in JSON
/// Handlebars HTML-escapes by default, but that's not the same as JSON escaping.
/// This function properly escapes JSON special characters.
fn escape_json_string(s: &str) -> String {
    s.chars()
        .flat_map(|c| match c {
            '"' => vec!['\\', '"'],
            '\\' => vec!['\\', '\\'],
            '\n' => vec!['\\', 'n'],
            '\r' => vec!['\\', 'r'],
            '\t' => vec!['\\', 't'],
            '\x08' => vec!['\\', 'b'],
            '\x0C' => vec!['\\', 'f'],
            c if c.is_control() => {
                // Escape other control characters as \uXXXX
                format!("\\u{:04x}", c as u32).chars().collect()
            }
            c => vec![c],
        })
        .collect()
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
    let filled = (progress_percent * bar_len) / 100;
    let empty = bar_len - filled;
    let progress_bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));

    // Prepare template data with JSON-escaped strings
    // SECURITY: All user-controlled strings must be JSON-escaped to prevent
    // malformed JSON from special characters like newlines, quotes, backslashes
    let mut data = std::collections::HashMap::new();
    data.insert("feature_name", escape_json_string(&feature.description));
    data.insert(
        "feature_category",
        escape_json_string(&capitalize_first(&feature.category)),
    );
    data.insert("project_name", escape_json_string(&project_name));
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

    handlebars
        .render_template(template_content, &data)
        .context("Failed to render webhook template")
}

fn extract_json_id(json: &str) -> Option<String> {
    // Parse JSON properly to extract the top-level "id" field
    // This is more robust than string matching and handles edge cases correctly
    let parsed: serde_json::Value = serde_json::from_str(json).ok()?;
    parsed.get("id")?.as_str().map(|s| s.to_string())
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
