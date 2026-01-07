//! Token usage statistics from OpenCode
//!
//! Handles fetching and parsing token/cost statistics from OpenCode sessions.

use std::process::Command;

/// Token usage statistics from OpenCode
#[derive(Debug, Default, Clone)]
pub struct TokenStats {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_cost: f64,
}

/// Fetch token stats for the current project by running `opencode stats`
pub fn fetch_token_stats() -> Option<TokenStats> {
    let output = Command::new("opencode")
        .args(["stats", "--project", ""])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_token_stats(&stdout)
}

fn parse_token_stats(output: &str) -> Option<TokenStats> {
    let mut stats = TokenStats::default();

    for line in output.lines() {
        let line = line.trim();

        // Parse lines like "Input tokens: 123,456" or "Total input: 123456"
        if line.to_lowercase().contains("input") && line.contains("token") {
            if let Some(num) = extract_number(line) {
                stats.input_tokens = num;
            }
        }

        // Parse lines like "Output tokens: 78,901"
        if line.to_lowercase().contains("output") && line.contains("token") {
            if let Some(num) = extract_number(line) {
                stats.output_tokens = num;
            }
        }

        // Parse cost lines like "Total cost: $1.23" or "Cost: $0.05"
        if line.to_lowercase().contains("cost") {
            if let Some(cost) = extract_cost(line) {
                stats.total_cost = cost;
            }
        }
    }

    if stats.input_tokens > 0 || stats.output_tokens > 0 {
        Some(stats)
    } else {
        None
    }
}

fn extract_number(s: &str) -> Option<u64> {
    // Find sequences of digits, removing commas
    let num_str: String = s
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == ',')
        .collect::<String>()
        .replace(',', "");

    // Take the last number in the string (usually the value after the label)
    num_str
        .split_whitespace()
        .last()?
        .parse()
        .ok()
        .or_else(|| num_str.parse().ok())
}

fn extract_cost(s: &str) -> Option<f64> {
    // Find pattern like $1.23 or 1.23
    for part in s.split_whitespace() {
        let cleaned = part.trim_start_matches('$').trim_end_matches(',');
        if let Ok(cost) = cleaned.parse::<f64>() {
            return Some(cost);
        }
    }
    None
}
