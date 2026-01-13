//! Reusable TUI components for the fullscreen interfaces
//!
//! This module provides building-block components for consistent UX:
//! - ToggleSwitch: Boolean [ON]/[OFF] toggles
//! - ProgressIndicator: Step wizard progress display
//! - FieldRow: Config field with proper styling
//! - ModeCard: Setup wizard mode selection card

#![allow(dead_code)]

use iocraft::prelude::*;

// ============================================================================
// Toggle Switch Component
// ============================================================================

#[derive(Default, Props)]
pub struct ToggleSwitchProps {
    pub value: bool,
    pub is_active: bool,
}

#[component]
pub fn ToggleSwitch(props: &ToggleSwitchProps) -> impl Into<AnyElement<'static>> {
    let on_color = if props.value {
        Color::Cyan
    } else {
        Color::DarkGrey
    };
    let off_color = if !props.value {
        Color::Cyan
    } else {
        Color::DarkGrey
    };

    element! {
        View(flex_direction: FlexDirection::Row) {
            Text(
                content: "[ON]",
                color: on_color,
                weight: if props.value { Weight::Bold } else { Weight::Normal },
            )
            Text(content: " ")
            Text(
                content: "[OFF]",
                color: off_color,
                weight: if !props.value { Weight::Bold } else { Weight::Normal },
            )
        }
    }
}

// ============================================================================
// Progress Indicator Component
// ============================================================================

#[derive(Default, Props)]
pub struct ProgressIndicatorProps<'a> {
    pub current_step: usize,
    pub total_steps: usize,
    pub step_label: &'a str,
}

#[component]
pub fn ProgressIndicator<'a>(props: &ProgressIndicatorProps<'a>) -> impl Into<AnyElement<'a>> {
    let dots: String = (1..=props.total_steps)
        .map(|i| {
            if i <= props.current_step {
                "●"
            } else {
                "○"
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    element! {
        View(flex_direction: FlexDirection::Row) {
            Text(
                content: format!("Step {} of {}", props.current_step, props.total_steps),
                color: Color::Grey,
            )
            Text(content: "  ")
            Text(content: dots, color: Color::Cyan)
            Text(content: format!("  {}", props.step_label), color: Color::White)
        }
    }
}

// ============================================================================
// Section Header Component
// ============================================================================

#[derive(Default, Props)]
pub struct SectionHeaderProps<'a> {
    pub title: &'a str,
}

#[component]
pub fn SectionHeader<'a>(props: &SectionHeaderProps<'a>) -> impl Into<AnyElement<'a>> {
    let separator_len = 36usize.saturating_sub(props.title.len());
    let separator = "─".repeat(separator_len);

    element! {
        View(flex_direction: FlexDirection::Row, margin_top: 1, margin_bottom: 1) {
            Text(content: "── ", color: Color::DarkGrey)
            Text(content: props.title.to_uppercase(), color: Color::Grey, weight: Weight::Bold)
            Text(content: format!(" {}", separator), color: Color::DarkGrey)
        }
    }
}

// ============================================================================
// Mode Card Component (for setup wizard)
// ============================================================================

#[derive(Default, Props)]
pub struct ModeCardProps<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub is_selected: bool,
}

#[component]
pub fn ModeCard<'a>(props: &ModeCardProps<'a>) -> impl Into<AnyElement<'a>> {
    let border_color = if props.is_selected {
        Color::Cyan
    } else {
        Color::DarkGrey
    };

    let title_color = if props.is_selected {
        Color::White
    } else {
        Color::Grey
    };

    element! {
        View(
            border_style: BorderStyle::Round,
            border_color: border_color,
            padding: 1,
            width: 38,
            flex_direction: FlexDirection::Column,
        ) {
            Text(content: props.title, color: title_color, weight: Weight::Bold)
            View(margin_top: 1) {
                Text(content: props.description, color: Color::DarkGrey)
            }
        }
    }
}
