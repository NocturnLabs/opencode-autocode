use crate::tui::validation::SpecAction;
use crate::validation::ValidationResult;
use anyhow::Result;
use iocraft::prelude::*;
use std::sync::{Arc, Mutex};

#[derive(Props, Default)]
struct SpecReviewProps {
    spec_text: String,
    validation: ValidationResult,
    #[allow(dead_code)]
    max_preview_lines: usize,
    result: Arc<Mutex<Option<SpecAction>>>,
}

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
            ("✅ Accept", SpecAction::Accept),
            ("✏️  Edit", SpecAction::Edit),
            ("📄 Save", SpecAction::SaveToFile),
            ("🔧 Refine", SpecAction::Refine),
            ("🔄 Regenerate", SpecAction::Regenerate),
            ("❌ Cancel", SpecAction::Cancel),
        ]
    } else {
        vec![
            ("⚠️  Accept anyway", SpecAction::Accept),
            ("✏️  Edit manually", SpecAction::Edit),
            ("📄 Save to file", SpecAction::SaveToFile),
            ("🔧 Refine", SpecAction::Refine),
            ("🔄 Regenerate", SpecAction::Regenerate),
            ("❌ Cancel", SpecAction::Cancel),
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

    element! {
        View(width, height, flex_direction: FlexDirection::Column) {
            // Header
            View(padding: 1, border_style: BorderStyle::Round, border_color: Color::Cyan) {
                Text(content: "📄 Specification Review", weight: Weight::Bold, color: Color::Cyan)
            }

            View(flex_grow: 1.0, flex_direction: FlexDirection::Row) {
                // Left side: Validation & Actions (~35% width)
                View(flex_grow: 0.65, flex_direction: FlexDirection::Column, padding: 1, border_style: BorderStyle::Single, border_color: Color::Grey) {
                    Text(content: "Status:", weight: Weight::Bold)
                    #(if props.validation.is_valid {
                        element! { Text(content: " ✅ VALID", color: Color::Green, weight: Weight::Bold) }
                    } else {
                        element! { Text(content: " ❌ INVALID", color: Color::Red, weight: Weight::Bold) }
                    })

                    #(if !props.validation.errors.is_empty() {
                        element! {
                            View(flex_direction: FlexDirection::Column) {
                                Text(content: "\nErrors:", color: Color::Red, weight: Weight::Bold)
                                #(props.validation.errors.iter().take(5).map(|err| {
                                    element! { Text(content: format!(" • {}", err), color: Color::Red) }
                                }))
                            }
                        }
                    } else {
                        element! { View() }
                    })

                    Text(content: "\nActions:", weight: Weight::Bold)
                    View(flex_direction: FlexDirection::Column, margin_top: 1) {
                        #(actions.iter().enumerate().map(|(i, (label, _))| {
                            let is_selected = i == selected_action.get();
                            element! {
                                View {
                                    Text(
                                        content: if is_selected { format!("▸ {}", label) } else { format!("  {}", label) },
                                        color: if is_selected { Color::Green } else { Color::White },
                                        weight: if is_selected { Weight::Bold } else { Weight::Normal },
                                    )
                                }
                            }
                        }))
                    }
                }

                // Right side: Preview (~65% width)
                View(flex_grow: 0.65, flex_direction: FlexDirection::Column, padding: 1, border_style: BorderStyle::Single, border_color: Color::Grey) {
                    Text(content: "Preview:", weight: Weight::Bold)
                    View(flex_grow: 1.0, margin_top: 1) {
                        Text(
                            content: preview_lines.iter().take(height as usize - 10).map(|s| s.to_string()).collect::<Vec<_>>().join("\n"),
                            color: Color::Grey,
                        )
                    }
                    Text(
                        content: format!("(Scroll: PageUp/PageDown | Line {}/{})", scroll_offset.get() + 1, spec_lines.len()),
                        color: Color::DarkGrey,
                    )
                }
            }

            // Footer
            View(padding: 1, border_style: BorderStyle::Single, border_color: Color::Grey) {
                Text(content: "↑/↓: Navigate  Enter: Select  q: Quit", color: Color::Grey)
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
