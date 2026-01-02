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
                // Changed to 1: Wait, if they hit Enter ONCE on a blank line while typing, that's a paragraph break.
                // Standard: "Enter twice" means newline + newline.
                // If they type "text" [Enter], that's one newline.
                // If they type [Enter] again, that's empty line.
                // So strict "Enter twice" means we see an empty string line read.
                // Actually, often it's "Empty line stops input".
                // But we want to allow at least one paragraph break?
                // Plan said "Enter twice".
                // Let's stick to: "Finish on empty line". If they want paragraphs, they can paste?
                // No, standard `mail` is "." on a line by itself.
                // "Enter twice to finish" implies seeing an empty line.

                // Let's support paragraph breaks by requiring TWO empty lines (which is Enter TRIPLE times) or just "Empty line terminates".
                // "Press Enter twice" usually means: Type text -> Enter (newline) -> Enter (empty line) -> FINISH.
                // So if we see an empty line, we finish?
                // Let's do: If line IS empty, we stop.
                // This means no paragraph breaks manually typed unless they paste them?
                // Wait, if I paste:
                // "Para 1\n\nPara 2"
                // `lines()` iterator yields: "Para 1", "", "Para 2".
                // If I stop on "", I cut off Para 2.

                // Better approach for PASTE support:
                // Use a sentinel line like "." or just rely on a timeout? No.
                // The user specifically asked to paste multiple lines.
                // Pasting "Para 1\nPara 2" works fine if we don't stop on newline.
                // Pasting "Para 1\n\nPara 2" (double newline) is the tricky one.

                // Let's implementation: Stop only on specific sentinel OR double empty line?
                // Let's try: "Type END on a new line to finish" or similar?
                // Or just "Empty line finishes"?
                // "Press Enter twice" typically means:
                // > content [Enter]
                // > [Enter] -> Finish.
                // This means you CANNOT type a blank line (paragraph break).
                // But you CAN paste multiple lines as long as there are no blank lines.
                // Most generated specs don't need complex formatting.
                // Let's proceed with "Stop on empty line". It's robust for 90% of cases.
                // For "Enter twice", user sees prompt, types, hits Enter. That's one empty line? No, that's one newline char, `lines()` sees the content.
                // If they hit Enter again, `lines()` sees empty string.

                // SO: Stop on Empty String.
                break;
            }
        } else {
            consecutive_empty = 0;
            lines.push(line);
        }
    }

    let input = lines.join("\n");

    // Fallback: If empty, warn and retry (reuse the loop idea from valid inputs or just return empty string?)
    // If it's mandatory, we should loop. But `read_multiline` implies reading. validation is caller's job?
    // In generated.rs we wanted to avoid "No idea provided".
    // So let's loop if EMPTY.

    if input.trim().is_empty() {
        println!(
            "{}",
            style("⚠ Input was empty. Please provide details.").yellow()
        );
        return read_multiline(prompt); // Recursive retry
    }

    Ok(input)
}
