//! Display functions for the autonomous runner

use crate::theming::{boxes, highlight, muted, primary, success, symbols};

/// Display the startup banner
pub fn display_banner(
    model: &str,
    max_iterations: usize,
    delay_seconds: u32,
    developer_mode: bool,
) {
    let width = 55;
    println!();
    println!(
        "{}{}{}",
        primary(boxes::TOP_LEFT),
        primary(boxes::line(width - 2)),
        primary(boxes::TOP_RIGHT)
    );
    println!(
        "{} {} OpenCode Autonomous Agent Runner {}{}",
        primary(boxes::VERTICAL),
        symbols::SPARKLE,
        " ".repeat(width - 38),
        primary(boxes::VERTICAL)
    );
    println!(
        "{}{}{}",
        primary(boxes::BOTTOM_LEFT),
        primary(boxes::line(width - 2)),
        primary(boxes::BOTTOM_RIGHT)
    );
    println!();

    println!(
        "  {} Project: {}",
        muted(symbols::BULLET),
        highlight(
            std::env::current_dir()
                .map(|p| p.display().to_string())
                .unwrap_or_default()
        )
    );
    println!(
        "  {} Max iterations: {}",
        muted(symbols::BULLET),
        highlight(if max_iterations == usize::MAX {
            "unlimited".to_string()
        } else {
            max_iterations.to_string()
        })
    );
    println!("  {} Model: {}", muted(symbols::BULLET), highlight(model));
    println!(
        "  {} Delay: {}s",
        muted(symbols::BULLET),
        highlight(delay_seconds)
    );
    if developer_mode {
        println!(
            "  {} Developer mode: {}",
            muted(symbols::BULLET),
            highlight("ENABLED")
        );
    }
    println!();
    println!(
        "  {} Sessions run in batch mode and continue automatically.",
        muted(symbols::INFO)
    );
    println!("  {} Press Ctrl+C to stop.", muted(symbols::INFO));
    println!();
}

/// Display the session header
pub fn display_session_header(iteration: usize) {
    println!();
    println!(
        "{}{}{}",
        primary(boxes::TOP_LEFT),
        primary(boxes::line(53)),
        primary(boxes::TOP_RIGHT)
    );
    println!(
        "{} {} Session {} {} {} {}",
        primary(boxes::VERTICAL),
        symbols::RUNNING,
        highlight(iteration),
        muted(symbols::ARROW),
        muted(chrono::Local::now().format("%H:%M:%S")),
        primary(boxes::VERTICAL)
    );
    println!(
        "{}{}{}",
        primary(boxes::BOTTOM_LEFT),
        primary(boxes::line(53)),
        primary(boxes::BOTTOM_RIGHT)
    );
    println!();
}

/// Display the final status when runner stops
pub fn display_final_status(passing: usize, total: usize, developer_mode: bool) {
    println!();
    println!(
        "{}{}{}",
        muted(boxes::TOP_LEFT),
        muted(boxes::line(53)),
        muted(boxes::TOP_RIGHT)
    );
    println!(
        "{}  Runner stopped {}",
        muted(boxes::VERTICAL),
        muted(boxes::VERTICAL)
    );
    println!(
        "{}{}{}",
        muted(boxes::BOTTOM_LEFT),
        muted(boxes::line(53)),
        muted(boxes::BOTTOM_RIGHT)
    );
    println!();

    if total > 0 {
        let status_symbol = if passing == total {
            success(symbols::SUCCESS)
        } else {
            muted(symbols::PENDING)
        };
        println!(
            "  {} Status: {} / {} tests passing",
            status_symbol,
            highlight(passing),
            total
        );
    }

    if developer_mode {
        println!(
            "\n  {} Debug log saved to: {}",
            symbols::INFO,
            highlight("opencode-debug.log")
        );
    }

    println!();
    println!(
        "  {} To resume: {}",
        muted(symbols::ARROW),
        highlight("opencode-autocode autonomous")
    );
    println!(
        "  {} To stop:   {}",
        muted(symbols::ARROW),
        highlight("touch .opencode-stop")
    );
}

/// Display token usage statistics after a session
pub fn display_token_stats(stats: &super::session::TokenStats) {
    println!();
    println!(
        "{}{}{}",
        muted(boxes::TOP_LEFT),
        muted(boxes::line(53)),
        muted(boxes::TOP_RIGHT)
    );
    println!(
        "{}  {} Token Usage (this project) {}",
        muted(boxes::VERTICAL),
        symbols::INFO,
        muted(boxes::VERTICAL)
    );
    println!(
        "{}{}{}",
        muted(boxes::BOTTOM_LEFT),
        muted(boxes::line(53)),
        muted(boxes::BOTTOM_RIGHT)
    );
    println!(
        "  Input:  {:>12} tokens",
        highlight(format_number(stats.input_tokens))
    );
    println!(
        "  Output: {:>12} tokens",
        highlight(format_number(stats.output_tokens))
    );
    println!(
        "  Total:  {:>12} tokens",
        highlight(format_number(stats.input_tokens + stats.output_tokens))
    );
    if stats.total_cost > 0.0 {
        println!(
            "  Cost:   {:>12}",
            success(format!("${:.4}", stats.total_cost))
        );
    }
    println!("{}", muted(boxes::line(55)));
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
