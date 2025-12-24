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
