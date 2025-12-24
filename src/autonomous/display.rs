//! Display functions for the autonomous runner

/// Display the startup banner
pub fn display_banner(
    model: &str,
    max_iterations: usize,
    delay_seconds: u32,
    developer_mode: bool,
) {
    println!("═══════════════════════════════════════════════════");
    println!("  OpenCode Autonomous Agent Runner");
    println!("═══════════════════════════════════════════════════");
    println!();
    println!(
        "Project directory: {}",
        std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_default()
    );
    println!(
        "Max iterations: {}",
        if max_iterations == usize::MAX {
            "unlimited".to_string()
        } else {
            max_iterations.to_string()
        }
    );
    println!("Model: {}", model);
    println!("Delay between sessions: {}s", delay_seconds);
    if developer_mode {
        println!("Developer mode: ENABLED");
    }
    println!();
    println!("Sessions will run in batch mode and continue automatically.");
    println!("Press Ctrl+C to stop.");
    println!();
}

/// Display the session header
pub fn display_session_header(iteration: usize) {
    println!();
    println!("═══════════════════════════════════════════════════");
    println!(
        "  Session {} - {}",
        iteration,
        chrono::Local::now().format("%H:%M:%S")
    );
    println!("═══════════════════════════════════════════════════");
    println!();
}

/// Display the final status when runner stops
pub fn display_final_status(passing: usize, total: usize, developer_mode: bool) {
    println!();
    println!("═══════════════════════════════════════════════════");
    println!("  Runner stopped");
    println!("═══════════════════════════════════════════════════");
    println!();

    if total > 0 {
        println!("Status: {} / {} tests passing", passing, total);
    }

    if developer_mode {
        println!("\n📋 Debug log saved to: opencode-debug.log");
    }

    println!();
    println!("To resume: opencode-autocode autonomous");
    println!("To stop:   touch .opencode-stop");
}

/// Display token usage statistics after a session
pub fn display_token_stats(stats: &super::session::TokenStats) {
    println!();
    println!("───────────────────────────────────────────────────");
    println!("  📊 Token Usage (this project)");
    println!("───────────────────────────────────────────────────");
    println!(
        "  Input:  {:>12} tokens",
        format_number(stats.input_tokens)
    );
    println!(
        "  Output: {:>12} tokens",
        format_number(stats.output_tokens)
    );
    println!(
        "  Total:  {:>12} tokens",
        format_number(stats.input_tokens + stats.output_tokens)
    );
    if stats.total_cost > 0.0 {
        println!("  Cost:   {:>12}", format!("${:.4}", stats.total_cost));
    }
    println!("───────────────────────────────────────────────────");
}

fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.insert(0, ',');
        }
        result.insert(0, c);
    }
    result
}

