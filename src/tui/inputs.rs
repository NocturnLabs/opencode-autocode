use anyhow::Result;
use console::style;
use std::io::{self, BufRead, Write};

/// Read multiline input from the user in the terminal.
///
/// Accumulates lines until the user enters two consecutive empty lines (or EOF).
pub fn read_multiline(prompt: &str) -> Result<String> {
    print!(
        "{}: ",
        style(format!("{} (Press Enter twice to finish)", prompt)).green()
    );
    let _ = io::stdout().flush();

    let stdin = io::stdin();
    let mut lines = Vec::new();
    let mut consecutive_empty = 0;

    for line_result in stdin.lock().lines() {
        let line = line_result?;

        if line.trim().is_empty() {
            consecutive_empty += 1;
            if consecutive_empty >= 1 {
             
                break;
            }
        } else {
            consecutive_empty = 0;
            lines.push(line);
        }
    }

    let input = lines.join("\n");


    if input.trim().is_empty() {
        println!(
            "{}",
            style("⚠ Input was empty. Please provide details.").yellow()
        );
        return read_multiline(prompt); // Recursive retry
    }

    Ok(input)
}
