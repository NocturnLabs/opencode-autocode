//! iocraft-based prompt helpers
//!
//! These functions provide interactive prompts using iocraft for styling,
//! serving as replacements for dialoguer prompts.

use anyhow::Result;
use iocraft::prelude::*;
use std::io::{self, BufRead, Write};

use crate::theming::symbols;
use crate::tui::theme::TuiTheme;

/// Print a success message
pub fn print_success(text: &str) {
    element! {
        View {
            Text(content: format!("{} ", symbols::SUCCESS), color: TuiTheme::SUCCESS, weight: Weight::Bold)
            Text(content: text, color: TuiTheme::SUCCESS)
        }
    }
    .print();
}

/// Print an error message
pub fn print_error(text: &str) {
    element! {
        View {
            Text(content: format!("{} ", symbols::ERROR), color: TuiTheme::ERROR, weight: Weight::Bold)
            Text(content: text, color: TuiTheme::ERROR)
        }
    }
    .print();
}

/// Print an info/hint message
pub fn print_info(text: &str) {
    element! {
        View {
            Text(content: format!("{} ", symbols::INFO), color: TuiTheme::ACCENT)
            Text(content: text, color: TuiTheme::MUTED)
        }
    }
    .print();
}

/// Interactive confirm prompt (replacement for dialoguer::Confirm)
pub fn confirm(prompt: &str, default: bool) -> Result<bool> {
    print!("{} [{}]: ", prompt, if default { "Y/n" } else { "y/N" });
    let _ = io::stdout().flush();

    let stdin = io::stdin();
    let input = stdin.lock().lines().next().transpose()?.unwrap_or_default();

    let result = match input.trim().to_lowercase().as_str() {
        "y" | "yes" => true,
        "n" | "no" => false,
        "" => default,
        _ => default,
    };

    Ok(result)
}

/// Interactive select prompt (replacement for dialoguer::Select)
pub fn select(prompt: &str, items: &[&str], default: usize) -> Result<usize> {
    println!("\n{}:", prompt);
    for (i, item) in items.iter().enumerate() {
        if i == default {
            element! {
                View {
                    Text(content: format!("  {} ", i + 1), color: TuiTheme::MUTED)
                    Text(content: *item, color: TuiTheme::ACCENT, weight: Weight::Bold)
                    Text(content: " (default)", color: TuiTheme::MUTED)
                }
            }
            .print();
        } else {
            element! {
                View {
                    Text(content: format!("  {} ", i + 1), color: TuiTheme::MUTED)
                    Text(content: *item, color: TuiTheme::PRIMARY)
                }
            }
            .print();
        }
    }

    print!("Select [1-{}] (default {}): ", items.len(), default + 1);
    let _ = io::stdout().flush();

    let stdin = io::stdin();
    let input = stdin.lock().lines().next().transpose()?.unwrap_or_default();

    let selection = if input.trim().is_empty() {
        default
    } else {
        input
            .trim()
            .parse::<usize>()
            .unwrap_or(default + 1)
            .saturating_sub(1)
    };

    Ok(selection.min(items.len().saturating_sub(1)))
}

/// Interactive text input (replacement for dialoguer::Input)
pub fn input(prompt: &str, default: Option<&str>) -> Result<String> {
    if let Some(def) = default {
        print!("{} [{}]: ", prompt, def);
    } else {
        print!("{}: ", prompt);
    }
    let _ = io::stdout().flush();

    let stdin = io::stdin();
    let input = stdin.lock().lines().next().transpose()?.unwrap_or_default();

    if input.trim().is_empty() {
        Ok(default.unwrap_or("").to_string())
    } else {
        Ok(input)
    }
}

/// Read multiline input
pub fn multiline_input(prompt: &str) -> Result<String> {
    element! {
        View {
            Text(content: format!("{} ", prompt), color: TuiTheme::ACCENT)
            Text(content: "(Press Enter 3 times to finish)", color: TuiTheme::MUTED)
        }
    }
    .print();
    println!();

    let stdin = io::stdin();
    let mut lines = Vec::new();
    let mut consecutive_empty = 0;

    for line_result in stdin.lock().lines() {
        let line = line_result?;

        if line.trim().is_empty() {
            consecutive_empty += 1;
            if consecutive_empty >= 2 {
                break;
            }
        } else {
            consecutive_empty = 0;
            lines.push(line);
        }
    }

    let input = lines.join("\n");

    if input.trim().is_empty() {
        print_error("Input was empty. Please provide details.");
        return multiline_input(prompt);
    }

    Ok(input)
}

/// Open text in external editor (replacement for dialoguer::Editor)
pub fn edit_in_editor(content: &str) -> Result<Option<String>> {
    use std::env;
    use std::fs;
    use std::process::Command;

    let editor = env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
    let temp_path = std::env::temp_dir().join(format!("iocraft_edit_{}.txt", std::process::id()));

    fs::write(&temp_path, content)?;

    let status = Command::new(&editor).arg(&temp_path).status()?;

    if status.success() {
        let edited = fs::read_to_string(&temp_path)?;
        let _ = fs::remove_file(&temp_path);
        if edited != content {
            Ok(Some(edited))
        } else {
            Ok(None)
        }
    } else {
        let _ = fs::remove_file(&temp_path);
        Ok(None)
    }
}
