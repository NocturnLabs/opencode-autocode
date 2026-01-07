use crate::db::sessions::SessionStats;
use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct DbStatsProps {
    pub total: usize,
    pub passing: usize,
    pub remaining: usize,
    pub session_stats: SessionStats,
}

#[component]
pub fn DbStatsView(props: &DbStatsProps) -> impl Into<AnyElement<'static>> {
    let passing_pct = if props.total > 0 {
        props.passing as f64 / props.total as f64 * 100.0
    } else {
        0.0
    };

    // Calculate bar width (total width 60 - 2 border - 2 padding = 56)
    let bar_width = (56.0 * passing_pct / 100.0) as i32;

    element! {
        View(
            flex_direction: FlexDirection::Column,
            border_style: BorderStyle::Round,
            border_color: Color::Magenta,
            width: 60,
            padding: 1,
            margin_top: 1,
            margin_bottom: 0,
        ) {
            // Header
            View(
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                margin_bottom: 1,
            ) {
                Text(content: "✨ Database Statistics", weight: Weight::Bold, color: Color::Magenta)
            }

            // Progress Bar Section
            View(
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                margin_bottom: 0,
            ) {
                Text(content: "Features Check:")
                Text(content: format!("{:.0}%", passing_pct))
            }

            // The Bar itself
            View(
                width: 100pct,
                height: 1,
                background_color: Color::DarkGrey,
                margin_bottom: 1,
            ) {
                View(
                    width: bar_width,
                    height: 100pct,
                    background_color: Color::Green,
                )
            }

            // Stats Table
            View(
                border_style: BorderStyle::Single,
                border_color: Color::DarkGrey,
                flex_direction: FlexDirection::Column,
                margin_bottom: 1,
            ) {
                // Row 1: Total
                View(flex_direction: FlexDirection::Row) {
                    View(width: 50pct, padding_left: 1) { Text(content: "• Total") }
                    View(width: 50pct, justify_content: JustifyContent::End, padding_right: 1) {
                        Text(content: props.total.to_string(), weight: Weight::Bold, color: Color::Cyan)
                    }
                }
                // Row 2: Passing
                 View(flex_direction: FlexDirection::Row) {
                    View(width: 50pct, padding_left: 1) { Text(content: "• Passing") }
                    View(width: 50pct, justify_content: JustifyContent::End, padding_right: 1) {
                        Text(content: props.passing.to_string(), color: Color::Green, weight: Weight::Bold)
                    }
                }
                // Row 3: Remaining
                 View(flex_direction: FlexDirection::Row) {
                    View(width: 50pct, padding_left: 1) { Text(content: "• Remaining") }
                    View(width: 50pct, justify_content: JustifyContent::End, padding_right: 1) {
                        Text(content: props.remaining.to_string(), color: Color::Yellow, weight: Weight::Bold)
                    }
                }
            }

            // Session Stats Section
            View(
                flex_direction: FlexDirection::Column,
            ) {
                View(margin_bottom: 0) {
                     Text(content: "Sessions:", weight: Weight::Bold)
                }

                View(flex_direction: FlexDirection::Row) {
                    View(width: 60pct, padding_left: 2) { Text(content: "Total") }
                    View(width: 40pct, justify_content: JustifyContent::End, padding_right: 1) {
                        Text(content: props.session_stats.total_sessions.to_string())
                    }
                }
                 View(flex_direction: FlexDirection::Row) {
                    View(width: 60pct, padding_left: 2) { Text(content: "Completed") }
                    View(width: 40pct, justify_content: JustifyContent::End, padding_right: 1) {
                        Text(content: props.session_stats.completed_sessions.to_string())
                    }
                }
                 View(flex_direction: FlexDirection::Row) {
                    View(width: 60pct, padding_left: 2) { Text(content: "Features Done") }
                    View(width: 40pct, justify_content: JustifyContent::End, padding_right: 1) {
                        Text(content: props.session_stats.total_features_completed.to_string())
                    }
                }
            }
        }
    }
}
