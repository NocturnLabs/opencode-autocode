//! Display functions for the autonomous runner

use crate::theming::{boxes, muted, symbols};

/// Display the startup banner
pub fn display_banner(
    model: &str,
    max_iterations: usize,
    delay_seconds: u32,
    developer_mode: bool,
) {
    let width = 60;

    println!();
    println!(
        "{}{}{}",
        crate::theming::accent(boxes::TOP_LEFT),
        crate::theming::accent(boxes::line(width - 2)),
        crate::theming::accent(boxes::TOP_RIGHT)
    );

    // Title with sparkle
    let title = "OpenCode Autonomous Agent";
    let padding = width - 4 - title.len() - 3; // -4 for borders/spaces, -3 for sparkle
    println!(
        "{} {} {} {}{}",
        crate::theming::accent(boxes::VERTICAL),
        symbols::SPARKLE,
        crate::theming::primary(title).bold(),
        " ".repeat(padding),
        crate::theming::accent(boxes::VERTICAL)
    );

    println!(
        "{}{}{}",
        crate::theming::accent(boxes::LEFT_T),
        crate::theming::accent(boxes::line(width - 2)),
        crate::theming::accent(boxes::RIGHT_T)
    );

    // Info rows
    let rows = vec![
        (
            "Project",
            std::env::current_dir()
                .map(|p| {
                    p.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string()
                })
                .unwrap_or_default(),
        ),
        ("Model", model.to_string()),
        (
            "Limit",
            if max_iterations == usize::MAX {
                "∞".to_string()
            } else {
                max_iterations.to_string()
            },
        ),
        ("Delay", format!("{}s", delay_seconds)),
    ];

    for (key, value) in rows {
        let padding = width - 6 - key.len() - value.len(); // -6: "| • " + ": " + " |"
        println!(
            "{} {} {}: {}{}{}",
            crate::theming::accent(boxes::VERTICAL),
            muted(symbols::BULLET),
            key,
            crate::theming::highlight(value),
            " ".repeat(padding),
            crate::theming::accent(boxes::VERTICAL)
        );
    }

    if developer_mode {
        let dev_msg = "DEVELOPER MODE ENABLED";
        let padding = width - 5 - dev_msg.len();
        println!(
            "{} {} {} {}{}",
            crate::theming::accent(boxes::VERTICAL),
            crate::theming::warning(symbols::WARNING),
            crate::theming::warning(dev_msg),
            " ".repeat(padding),
            crate::theming::accent(boxes::VERTICAL)
        );
    }

    println!(
        "{}{}{}",
        crate::theming::accent(boxes::BOTTOM_LEFT),
        crate::theming::accent(boxes::line(width - 2)),
        crate::theming::accent(boxes::BOTTOM_RIGHT)
    );
    println!();
}

/// Display the session header
pub fn display_session_header(iteration: usize) {
    let width = 60;

    println!();
    // Compact header style: ╭── Session 1 ── [12:00:00] ───╮
    let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
    let iter_str = format!("Session {}", iteration);
    let time_str = format!("[{}]", timestamp);

    // Calculate line lengths
    // Total width = 60
    // "╭─ " = 3 chars
    // " ─ " = 3 chars
    // " ─ " = 3 chars
    // "─╮" = 2 chars
    // Remaining for lines: 60 - 11 - iter_len - time_len

    let total_padding = width - 13 - iter_str.len() - time_str.len();
    let left_pad = total_padding / 3;
    let mid_pad = total_padding / 3;
    let right_pad = total_padding - left_pad - mid_pad;

    println!(
        "{}{} {} {} {} {} {}{}",
        crate::theming::accent(boxes::TOP_LEFT),
        crate::theming::accent(boxes::line(left_pad + 2)),
        crate::theming::primary(iter_str).bold(),
        crate::theming::accent(boxes::HORIZONTAL),
        muted(time_str),
        crate::theming::accent(boxes::HORIZONTAL),
        crate::theming::accent(boxes::line(right_pad + mid_pad + 2)), // Simplified distribution
        crate::theming::accent(boxes::TOP_RIGHT)
    );
    println!(
        "{}{}{}",
        crate::theming::accent(boxes::BOTTOM_LEFT),
        crate::theming::accent(boxes::line(width - 2)),
        crate::theming::accent(boxes::BOTTOM_RIGHT)
    );
}

/// Display the final status when runner stops
pub fn display_final_status(passing: usize, total: usize, developer_mode: bool) {
    println!();
    // Summary block
    println!();
    let status_str = if passing == total && total > 0 {
        format!(
            "{} All features passing! ({}/{})",
            symbols::SUCCESS,
            passing,
            total
        )
    } else {
        format!(
            "{} Progress: {}/{} passing",
            symbols::PENDING,
            passing,
            total
        )
    };

    let status_color = if passing == total && total > 0 {
        crate::theming::success
    } else {
        crate::theming::warning
    };

    println!("  {}", status_color(status_str).bold());

    if developer_mode {
        println!(
            "  {} Debug log: {}",
            muted(symbols::INFO),
            crate::theming::highlight("opencode-debug.log")
        );
    }

    println!();
    println!(
        "  {} Next steps:",
        crate::theming::primary(symbols::CHEVRON)
    );
    println!(
        "    {} Resume: {}",
        muted(symbols::BULLET),
        crate::theming::highlight("opencode-forger autonomous")
    );
    println!(
        "    {} Stop:   {}",
        muted(symbols::BULLET),
        crate::theming::highlight("touch .opencode-stop")
    );
    println!();
}

/// Display token usage statistics after a session
pub fn display_token_stats(stats: &super::stats::TokenStats) {
    let width = 60;

    // A simpler token stats block, integrated into the flow
    // ╭ Make it look like a connected section or a separate clear block

    println!(
        "{}{}{}",
        crate::theming::accent(boxes::TOP_LEFT),
        crate::theming::accent(boxes::line(width - 2)),
        crate::theming::accent(boxes::TOP_RIGHT)
    );

    println!(
        "{} {} Session Stats:",
        crate::theming::accent(boxes::VERTICAL),
        crate::theming::primary(symbols::LOCK) // Or some other symbol
    );

    let rows = vec![
        (
            "Input",
            format!("{} tokens", format_number(stats.input_tokens)),
        ),
        (
            "Output",
            format!("{} tokens", format_number(stats.output_tokens)),
        ),
        (
            "Total",
            format!(
                "{} tokens",
                format_number(stats.input_tokens + stats.output_tokens)
            ),
        ),
        ("Cost", format!("${:.4}", stats.total_cost)),
    ];

    for (key, value) in rows {
        let padding = width - 7 - key.len() - value.len();
        println!(
            "{}   {}: {}{}{}",
            crate::theming::accent(boxes::VERTICAL),
            muted(key),
            crate::theming::highlight(value),
            " ".repeat(padding),
            crate::theming::accent(boxes::VERTICAL)
        );
    }

    // Bottom border is handled by the next section or we close it here if it's standalone
    // Actually, display_token_stats is called INSIDE the loop usually.
    // Let's just close it for now to be safe.
    println!(
        "{}{}{}",
        crate::theming::accent(boxes::BOTTOM_LEFT),
        crate::theming::accent(boxes::line(width - 2)),
        crate::theming::accent(boxes::BOTTOM_RIGHT)
    );
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
