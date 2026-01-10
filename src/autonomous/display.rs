//! Display functions for the autonomous runner

use crate::theming::{boxes, muted, symbols, visual_width};

/// Display the startup banner and return the calculated width
pub fn display_banner(
    model: &str,
    max_iterations: usize,
    delay_seconds: u32,
    developer_mode: bool,
) -> usize {
    let title = "OpenCode Autonomous Agent";
    let dev_msg = "DEVELOPER MODE ENABLED";

    // Build info rows first to calculate width
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

    // Calculate dynamic width based on content
    // We use visual_width to handle multi-byte characters like ✨, ⚠, •, and ∞

    // Title Line: "│ ✨ Title │"
    // Intrinsic parts: │ (1) + space (1) + ✨ (2) + space (1) + len + space (1) + │ (1) = 7 + len
    let title_intrinsic_width = 7 + visual_width(title);

    // Info Rows: "│ • Key: value │"
    // Intrinsic parts: │ (1) + space (1) + • (1) + space (1) + k_len + ": " (2) + v_len + space (1) + │ (1) = 8 + k_len + v_len
    // Wait, let's re-count: │ (1) + " " (1) + • (1) + " " (1) + key + ": " (2) + value + " " (1) + │ (1) = 8 + k_len + v_len
    // Actually in the previous implementation I used 7. Let's be precise.
    // │(1) " "(1) •(1) " "(1) key(k) ": "(2) value(v) " "(1) │(1) = 1+1+1+1+k+2+v+1+1 = 9 + k + v
    // Let's re-examine the print statement: println!("{} {} {}: {}{}{}", VERTICAL, BULLET, key, highlight(value), " ".repeat(padding), VERTICAL);
    // Fixed parts: VERTICAL (1) + " " (1) + BULLET (1) + " " (1) + ":" (1) + " " (1) + VERTICAL (1) = 6
    // Variable parts: key + value + padding
    // Total width = 6 + visual_width(key) + visual_width(value) + padding
    // So intrinsic part (without padding) is 6 + visual_width(key) + visual_width(value)
    // To have at least 1 space of padding on the right, we want width >= 7 + visual_width(key) + visual_width(value)

    let max_row_intrinsic_width = rows
        .iter()
        .map(|(k, v)| 7 + visual_width(k) + visual_width(v))
        .max()
        .unwrap_or(0);

    // Dev Line: "│ ⚠ MESSAGE │"
    // Intrinsic: │(1) " "(1) ⚠(2) " "(1) msg(len) " "(1) │(1) = 7 + len
    let dev_intrinsic_width = if developer_mode {
        7 + visual_width(dev_msg)
    } else {
        0
    };

    // Title defines the baseline minimum width
    let width = title_intrinsic_width
        .max(max_row_intrinsic_width)
        .max(dev_intrinsic_width);

    println!(
        "{}{}{}",
        crate::theming::accent(boxes::TOP_LEFT),
        crate::theming::accent(boxes::line(width - 2)),
        crate::theming::accent(boxes::TOP_RIGHT)
    );

    // Title with sparkle
    let padding = width.saturating_sub(7).saturating_sub(visual_width(title));
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

    for (key, value) in rows {
        let padding = width
            .saturating_sub(7)
            .saturating_sub(visual_width(key))
            .saturating_sub(visual_width(&value));
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
        let padding = width
            .saturating_sub(7)
            .saturating_sub(visual_width(dev_msg));
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

    width
}

/// Display the session header with a specific width
pub fn display_session_header(iteration: usize, width: usize) {
    println!();
    let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
    let iter_str = format!("Session {}", iteration);
    let time_str = format!("[{}]", timestamp);

    // Calculate line lengths
    // Total width = width
    // 1 (╭) + 2 (──) + 1 (space) + iter_len + 1 (space) + 1 (─) + 1 (space) + time_len + 1 (space) + 1 (──) + 1 (╮)
    // Fixed parts: ╭── (3) + " " (1) + " " (1) + ─ (1) + " " (1) + " " (1) + ──╮ (3) = 11
    // Actually let's use a simpler calculation:
    let fixed_overhead = 11 + visual_width(&iter_str) + visual_width(&time_str);
    let extra_lines = width.saturating_sub(fixed_overhead);
    let left_line_len = extra_lines / 2;
    let right_line_len = extra_lines - left_line_len;

    println!(
        "{}{} {} {} {} {}{}",
        crate::theming::accent(boxes::TOP_LEFT),
        crate::theming::accent(boxes::line(left_line_len + 2)),
        crate::theming::primary(iter_str).bold(),
        crate::theming::accent(boxes::HORIZONTAL),
        muted(time_str),
        crate::theming::accent(boxes::line(right_line_len + 2)),
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
        crate::theming::highlight("opencode-forger vibe")
    );
    println!(
        "    {} Stop:   {}",
        muted(symbols::BULLET),
        crate::theming::highlight("touch .opencode-stop")
    );
    println!();
}

/// Display token usage statistics after a session
pub fn display_token_stats(stats: &super::stats::TokenStats, width: usize) {
    println!(
        "{}{}{}",
        crate::theming::accent(boxes::TOP_LEFT),
        crate::theming::accent(boxes::line(width - 2)),
        crate::theming::accent(boxes::TOP_RIGHT)
    );

    println!(
        "{} {} Session Stats:",
        crate::theming::accent(boxes::VERTICAL),
        crate::theming::primary(symbols::LOCK)
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
        // │(1) + space(1) + space(1) + space(1) + key + ": "(2) + value + padding + space(1) + │(1) = 7 + key + value
        let padding = width
            .saturating_sub(7)
            .saturating_sub(visual_width(key))
            .saturating_sub(visual_width(&value));
        println!(
            "{}   {}: {}{}{}",
            crate::theming::accent(boxes::VERTICAL),
            muted(key),
            crate::theming::highlight(value),
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
