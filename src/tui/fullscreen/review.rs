use crate::theming::symbols;
use crate::tui::theme::TuiTheme;
use crate::tui::validation::SpecAction;
use crate::validation::ValidationResult;
use anyhow::Result;
use iocraft::prelude::*;
use std::sync::{Arc, Mutex};
use unicode_width::UnicodeWidthChar;

#[derive(Props, Default)]
struct SpecReviewProps {
    spec_text: String,
    validation: ValidationResult,
    max_preview_lines: usize,
    result: Arc<Mutex<Option<SpecAction>>>,
}

/// Wrap a single line to the specified display width.
fn wrap_line(line: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return vec![String::new()];
    }

    let mut lines = Vec::new();
    let mut current = String::new();
    let mut current_width = 0usize;

    for ch in line.chars() {
        let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);
        if current_width + ch_width > max_width && !current.is_empty() {
            lines.push(current);
            current = String::new();
            current_width = 0;
        }

        current.push(ch);
        current_width += ch_width;
    }

    lines.push(current);
    lines
}

/// Build the preview text with wrapping and line limits.
fn build_preview(lines: &[&str], max_width: usize, max_lines: usize) -> String {
    if max_lines == 0 {
        return String::new();
    }

    let mut wrapped = Vec::new();
    let width = max_width.max(1);

    for line in lines {
        wrapped.extend(wrap_line(line, width));
        if wrapped.len() >= max_lines {
            break;
        }
    }

    wrapped.truncate(max_lines);
    wrapped.join("\n")
}

/// Render the fullscreen specification review experience.
#[component]
fn SpecReview(props: &SpecReviewProps, mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let (width, height) = hooks.use_terminal_size();
    let mut system = hooks.use_context_mut::<SystemContext>();
    let mut selected_action = hooks.use_state(|| {
        if props.validation.is_valid {
            0usize
        } else {
            1usize
        }
    });
    let mut should_exit = hooks.use_state(|| false);
    let mut scroll_offset = hooks.use_state(|| 0usize);

    let actions = if props.validation.is_valid {
        vec![
            (format!("{} Accept", symbols::SUCCESS), SpecAction::Accept),
            (format!("{} Edit", symbols::CHEVRON), SpecAction::Edit),
            (format!("{} Save", symbols::BULLET), SpecAction::SaveToFile),
            (format!("{} Refine", symbols::ARROW), SpecAction::Refine),
            (
                format!("{} Regenerate", symbols::RUNNING),
                SpecAction::Regenerate,
            ),
            (format!("{} Cancel", symbols::ERROR), SpecAction::Cancel),
        ]
    } else {
        vec![
            (
                format!("{} Accept anyway", symbols::WARNING),
                SpecAction::Accept,
            ),
            (
                format!("{} Edit manually", symbols::CHEVRON),
                SpecAction::Edit,
            ),
            (
                format!("{} Save to file", symbols::BULLET),
                SpecAction::SaveToFile,
            ),
            (format!("{} Refine", symbols::ARROW), SpecAction::Refine),
            (
                format!("{} Regenerate", symbols::RUNNING),
                SpecAction::Regenerate,
            ),
            (format!("{} Cancel", symbols::ERROR), SpecAction::Cancel),
        ]
    };

    let actions_len = actions.len();

    let result = props.result.clone();
    let actions_for_events = actions.clone();

    hooks.use_terminal_events(move |event| match event {
        TerminalEvent::Key(KeyEvent { code, kind, .. }) if kind != KeyEventKind::Release => {
            match code {
                KeyCode::Up | KeyCode::Char('k') => {
                    if selected_action.get() > 0 {
                        selected_action.set(selected_action.get() - 1);
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if selected_action.get() < actions_len - 1 {
                        selected_action.set(selected_action.get() + 1);
                    }
                }
                KeyCode::PageUp => {
                    scroll_offset.set(scroll_offset.get().saturating_sub(10));
                }
                KeyCode::PageDown => {
                    scroll_offset.set(scroll_offset.get() + 10);
                }
                KeyCode::Enter => {
                    let mut res = result.lock().unwrap();
                    *res = Some(actions_for_events[selected_action.get()].1);
                    should_exit.set(true);
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    should_exit.set(true);
                }
                _ => {}
            }
        }
        _ => {}
    });

    if should_exit.get() {
        system.exit();
    }

    let spec_lines: Vec<_> = props.spec_text.lines().collect();
    let preview_lines = &spec_lines[scroll_offset.get().min(spec_lines.len())..];

    let total_width = width as u32;
    let min_left_width = 24u32;
    let max_left_width = total_width.saturating_sub(24);
    let ideal_left_width = total_width.saturating_mul(32) / 100;
    let left_width = if max_left_width <= min_left_width {
        max_left_width.max(12)
    } else {
        ideal_left_width.clamp(min_left_width, max_left_width)
    };
    let right_width = total_width.saturating_sub(left_width + 2);
    let preview_width = right_width.saturating_sub(4) as usize;
    let preview_height = height.saturating_sub(8) as usize;
    let preview_limit = props.max_preview_lines.min(preview_height);
    let preview_text = build_preview(preview_lines, preview_width, preview_limit);

    element! {
        View(width, height, flex_direction: FlexDirection::Column) {
            // Header
            View(
                padding: 1,
                border_style: BorderStyle::Round,
                border_color: TuiTheme::BORDER,
            ) {
                Text(
                    content: format!("{} Specification Review", symbols::INFO),
                    weight: Weight::Bold,
                    color: TuiTheme::ACCENT,
                )
            }

            View(
                flex_grow: 1.0,
                flex_direction: FlexDirection::Row,
                margin_top: 1,
            ) {
                // Left side: Validation & Actions (~35% width)
                View(
                    width: left_width,
                    flex_direction: FlexDirection::Column,
                    padding: 1,
                    border_style: BorderStyle::Single,
                    border_color: TuiTheme::BORDER,
                    margin_right: 1,
                ) {
                    Text(content: "Status", weight: Weight::Bold, color: TuiTheme::PRIMARY)
                    #(if props.validation.is_valid {
                        element! {
                            Text(
                                content: format!("{} VALID", symbols::SUCCESS),
                                color: TuiTheme::SUCCESS,
                                weight: Weight::Bold,
                            )
                        }
                    } else {
                        element! {
                            Text(
                                content: format!("{} INVALID", symbols::ERROR),
                                color: TuiTheme::ERROR,
                                weight: Weight::Bold,
                            )
                        }
                    })

                    #(if !props.validation.errors.is_empty() {
                        element! {
                            View(margin_top: 1, flex_direction: FlexDirection::Column) {
                                Text(content: "Errors", color: TuiTheme::ERROR, weight: Weight::Bold)
                                #(props.validation.errors.iter().take(5).map(|err| {
                                    element! {
                                        Text(content: format!("{} {}", symbols::BULLET, err), color: TuiTheme::ERROR)
                                    }
                                }))
                            }
                        }
                    } else {
                        element! { View() }
                    })

                    View(margin_top: 1) {
                        Text(content: "Actions", weight: Weight::Bold, color: TuiTheme::PRIMARY)
                    }
                    View(flex_direction: FlexDirection::Column, margin_top: 1) {
                        #(actions.iter().enumerate().map(|(i, (label, _))| {
                            let is_selected = i == selected_action.get();
                            element! {
                                View {
                                    Text(
                                        content: if is_selected {
                                            format!("{} {}", symbols::CHEVRON, label)
                                        } else {
                                            format!("  {}", label)
                                        },
                                        color: if is_selected { TuiTheme::ACCENT } else { TuiTheme::PRIMARY },
                                        weight: if is_selected { Weight::Bold } else { Weight::Normal },
                                    )
                                }
                            }
                        }))
                    }
                }

                // Right side: Preview (~65% width)
                View(
                    width: right_width,
                    flex_direction: FlexDirection::Column,
                    padding: 1,
                    border_style: BorderStyle::Single,
                    border_color: TuiTheme::BORDER,
                ) {
                    Text(content: "Preview", weight: Weight::Bold, color: TuiTheme::PRIMARY)
                    View(flex_grow: 1.0, margin_top: 1) {
                        Text(content: preview_text, color: TuiTheme::PRIMARY)
                    }
                    Text(
                        content: format!(
                            "{} Scroll: PageUp/PageDown | Line {}/{}",
                            symbols::INFO,
                            scroll_offset.get() + 1,
                            spec_lines.len()
                        ),
                        color: TuiTheme::MUTED,
                    )
                }
            }

            // Footer
            View(
                padding: 1,
                border_style: BorderStyle::Single,
                border_color: TuiTheme::BORDER,
            ) {
                Text(
                    content: format!("{} Navigate  Enter: Select  q: Quit", symbols::INFO),
                    color: TuiTheme::MUTED,
                )
            }
        }
    }
}

/// Run the fullscreen spec review and return the selected action
pub fn run_fullscreen_spec_review(
    spec_text: &str,
    validation: ValidationResult,
    max_preview_lines: usize,
) -> Result<SpecAction> {
    let result = Arc::new(Mutex::new(None));
    let result_clone = result.clone();

    smol::block_on(async {
        element! {
            SpecReview(
                spec_text: spec_text.to_string(),
                validation: validation,
                max_preview_lines: max_preview_lines,
                result: result_clone,
            )
        }
        .fullscreen()
        .await
    })?;

    let res = result.lock().unwrap().take();
    Ok(res.unwrap_or(SpecAction::Cancel))
}
