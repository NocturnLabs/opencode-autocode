//! Display functions for the autonomous runner

use crate::theming::{accent, boxes, highlight, muted, symbols, visual_width, warning};
use terminal_size::{terminal_size, Width};
use unicode_width::UnicodeWidthChar;

fn wrap_to_width(text: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return vec![text.to_string()];
    }

    let mut lines = Vec::new();
    let mut current = String::new();
    let mut width = 0usize;

    for ch in text.chars() {
        let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);
        if width + ch_width > max_width && !current.is_empty() {
            lines.push(current);
            current = String::new();
            width = 0;
        }
        current.push(ch);
        width += ch_width;
    }

    if !current.is_empty() {
        lines.push(current);
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

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

    // Calculate dynamic width based on content so long values don't overflow.
    let title_content = format!("{} {}", symbols::SPARKLE, title);

    let max_row_width = rows
        .iter()
        .map(|(key, value)| {
            let content = format!("{} {}: {}", symbols::BULLET, key, value);
            visual_width(&content)
        })
        .max()
        .unwrap_or(0);

    let dev_width = if developer_mode {
        visual_width(&format!("{} {}", symbols::WARNING, dev_msg))
    } else {
        0
    };

    let preferred_inner = visual_width(&title_content)
        .max(max_row_width)
        .max(dev_width)
        .max(10);

    let terminal_inner = terminal_size()
        .map(|(Width(w), _)| w as usize)
        .unwrap_or(usize::MAX)
        .saturating_sub(4);
    let available_inner = terminal_inner.max(10);
    let inner_width = preferred_inner.min(available_inner);
    let width = inner_width + 4;

    println!(
        "{}{}{}",
        accent(boxes::TOP_LEFT),
        accent(boxes::line(width - 2)),
        accent(boxes::TOP_RIGHT)
    );

    // Title with sparkle
    let title_padding = inner_width.saturating_sub(visual_width(&title_content));
    let title_styled = format!("{} {}", accent(symbols::SPARKLE), highlight(title));
    println!(
        "{} {}{} {}",
        accent(boxes::VERTICAL),
        title_styled,
        " ".repeat(title_padding),
        accent(boxes::VERTICAL)
    );

    println!(
        "{}{}{}",
        accent(boxes::TOP_LEFT),
        accent(boxes::line(width - 2)),
        accent(boxes::TOP_RIGHT)
    );

    for (key, value) in rows {
        let prefix = format!("{} {}: ", symbols::BULLET, key);
        let prefix_width = visual_width(&prefix);
        let available_value_width = inner_width.saturating_sub(prefix_width).max(1);
        let segments = wrap_to_width(&value, available_value_width);
        let indent = " ".repeat(prefix_width);

        for (idx, segment) in segments.iter().enumerate() {
            let plain = if idx == 0 {
                format!("{}{}", prefix, segment)
            } else {
                format!("{}{}", indent, segment)
            };

            let styled = if idx == 0 {
                format!("{} {}: {}", muted(symbols::BULLET), key, highlight(segment))
            } else {
                format!("{}{}", indent, highlight(segment))
            };

            let padding = inner_width.saturating_sub(visual_width(&plain));
            println!(
                "{} {}{} {}",
                accent(boxes::VERTICAL),
                styled,
                " ".repeat(padding),
                accent(boxes::VERTICAL)
            );
        }
    }

    if developer_mode {
        let dev_content = format!("{} {}", symbols::WARNING, dev_msg);
        let padding = inner_width.saturating_sub(visual_width(&dev_content));
        let dev_styled = format!("{} {}", warning(symbols::WARNING), warning(dev_msg));
        println!(
            "{} {}{} {}",
            accent(boxes::VERTICAL),
            dev_styled,
            " ".repeat(padding),
            accent(boxes::VERTICAL)
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
        accent(boxes::BOTTOM_LEFT),
        accent(boxes::line(width - 2)),
        accent(boxes::BOTTOM_RIGHT)
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
