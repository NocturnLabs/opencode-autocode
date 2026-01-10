use crate::config::Config;
use anyhow::Result;
use iocraft::prelude::*;
use std::sync::{Arc, Mutex};

#[derive(Props, Default)]
struct ConfigEditorProps {
    config: Arc<Mutex<Config>>,
    #[allow(dead_code)]
    available_models: Vec<String>,
}

/// Format a Vec<String> as a comma-separated string for display
fn format_array(items: &[String]) -> String {
    if items.is_empty() {
        "(empty)".to_string()
    } else {
        items.join(", ")
    }
}

/// Parse a comma-separated string into Vec<String>
fn parse_array(input: &str) -> Vec<String> {
    input
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

#[component]
fn ConfigEditor(props: &ConfigEditorProps, mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let (width, height) = hooks.use_terminal_size();
    let mut system = hooks.use_context_mut::<SystemContext>();
    let mut selected_section = hooks.use_state(|| 0usize);
    let mut selected_field = hooks.use_state(|| 0usize);
    let mut is_editing = hooks.use_state(|| false);
    let mut edit_buffer = hooks.use_state(String::new);
    let mut should_exit = hooks.use_state(|| false);
    let global_mcp_tools = hooks
        .use_state(|| crate::config::mcp_loader::load_global_mcp_servers().unwrap_or_default());

    let config_arc = props.config.clone();
    // Only show functional config sections
    let sections = vec![
        "Models",        // 0
        "Generation",    // 1
        "Autonomous",    // 2
        "Agent",         // 3
        "Recovery",      // 4
        "MCP",           // 5
        "Security",      // 6
        "UI",            // 7
        "Notifications", // 8
        "Features",      // 9
        "Scaffolding",   // 10
        "Conductor",     // 11
        "Paths",         // 12
        "Communication", // 13
    ];
    let sections_len = sections.len();

    // Get fields for the current section
    let fields = {
        let config = config_arc.lock().unwrap();
        match selected_section.get() {
            0 => vec![
                (
                    "Autonomous Model".to_string(),
                    config.models.autonomous.clone(),
                ),
                (
                    "Default/Spec Model".to_string(),
                    config.models.default.clone(),
                ),
                (
                    "Reasoning Model".to_string(),
                    config.models.reasoning.clone(),
                ),
                (
                    "Enhancement Model".to_string(),
                    config.models.enhancement.clone(),
                ),
                ("Fixer Model".to_string(), config.models.fixer.clone()),
            ],
            1 => vec![
                (
                    "Complexity".to_string(),
                    config.generation.complexity.clone(),
                ),
                (
                    "Subagents".to_string(),
                    config.generation.enable_subagents.to_string(),
                ),
                (
                    "Security Section".to_string(),
                    config.generation.include_security_section.to_string(),
                ),
                (
                    "Testing Section".to_string(),
                    config.generation.include_testing_strategy.to_string(),
                ),
                (
                    "DevOps Section".to_string(),
                    config.generation.include_devops_section.to_string(),
                ),
                (
                    "Accessibility".to_string(),
                    config.generation.include_accessibility.to_string(),
                ),
                (
                    "Future Enhancements".to_string(),
                    config.generation.include_future_enhancements.to_string(),
                ),
            ],
            2 => vec![
                (
                    "Max Iterations".to_string(),
                    config.autonomous.max_iterations.to_string(),
                ),
                (
                    "Delay (sec)".to_string(),
                    config.autonomous.delay_between_sessions.to_string(),
                ),
                (
                    "Timeout (min)".to_string(),
                    config.autonomous.session_timeout_minutes.to_string(),
                ),
                (
                    "Idle Timeout (sec)".to_string(),
                    config.autonomous.idle_timeout_seconds.to_string(),
                ),
                (
                    "Auto Commit".to_string(),
                    config.autonomous.auto_commit.to_string(),
                ),
                ("Log Level".to_string(), config.autonomous.log_level.clone()),
            ],
            3 => vec![
                (
                    "Max Retries".to_string(),
                    config.agent.max_retry_attempts.to_string(),
                ),
                (
                    "Max Research".to_string(),
                    config.agent.max_research_attempts.to_string(),
                ),
                (
                    "Verify Sample Size".to_string(),
                    config.agent.verification_sample_size.to_string(),
                ),
                (
                    "Single Focus".to_string(),
                    config.agent.single_feature_focus.to_string(),
                ),
            ],
            4 => vec![
                (
                    "Enabled".to_string(),
                    config.alternative_approaches.enabled.to_string(),
                ),
                (
                    "Approaches".to_string(),
                    config.alternative_approaches.num_approaches.to_string(),
                ),
                (
                    "Retry Threshold".to_string(),
                    config.alternative_approaches.retry_threshold.to_string(),
                ),
                (
                    "Cache Results".to_string(),
                    config.alternative_approaches.cache_results.to_string(),
                ),
                (
                    "Cache Dir".to_string(),
                    config.alternative_approaches.cache_dir.clone(),
                ),
            ],
            5 => {
                let mut tools = vec![
                    ("OsGrep".to_string(), config.mcp.prefer_osgrep.to_string()),
                    (
                        "Sequential".to_string(),
                        config.mcp.use_sequential_thinking.to_string(),
                    ),
                ];
                for tool in global_mcp_tools.read().iter() {
                    let enabled = config.mcp.required_tools.contains(tool);
                    tools.push((tool.clone(), enabled.to_string()));
                }
                tools
            }
            6 => vec![
                (
                    "Allowlist File".to_string(),
                    config.security.allowlist_file.clone(),
                ),
                (
                    "Strict".to_string(),
                    config.security.enforce_allowlist.to_string(),
                ),
                (
                    "Blocked Patterns".to_string(),
                    format_array(&config.security.blocked_patterns),
                ),
            ],
            7 => vec![
                (
                    "Colored Output".to_string(),
                    config.ui.colored_output.to_string(),
                ),
                ("Verbose".to_string(), config.ui.verbose.to_string()),
                (
                    "Show Progress".to_string(),
                    config.ui.show_progress.to_string(),
                ),
                (
                    "Spec Preview Lines".to_string(),
                    config.ui.spec_preview_lines.to_string(),
                ),
            ],
            8 => vec![
                (
                    "Webhook Enabled".to_string(),
                    config.notifications.webhook_enabled.to_string(),
                ),
                (
                    "Webhook URL".to_string(),
                    config.notifications.webhook_url.clone().unwrap_or_default(),
                ),
            ],
            9 => vec![
                (
                    "Categories".to_string(),
                    format_array(&config.features.categories),
                ),
                (
                    "Priorities".to_string(),
                    format_array(&config.features.priorities),
                ),
                (
                    "Require Verify Cmd".to_string(),
                    config.features.require_verification_command.to_string(),
                ),
                (
                    "Narrow Min Steps".to_string(),
                    config.features.narrow_test_min_steps.to_string(),
                ),
                (
                    "Narrow Max Steps".to_string(),
                    config.features.narrow_test_max_steps.to_string(),
                ),
                (
                    "Comprehensive Min Steps".to_string(),
                    config.features.comprehensive_test_min_steps.to_string(),
                ),
            ],
            10 => vec![
                (
                    "Git Init".to_string(),
                    config.scaffolding.git_init.to_string(),
                ),
                (
                    "Output Dir".to_string(),
                    config.scaffolding.output_dir.clone(),
                ),
                (
                    "Create OpenCode Dir".to_string(),
                    config.scaffolding.create_opencode_dir.to_string(),
                ),
                (
                    "Create Scripts Dir".to_string(),
                    config.scaffolding.create_scripts_dir.to_string(),
                ),
            ],
            11 => vec![
                (
                    "Context Dir".to_string(),
                    config.conductor.context_dir.clone(),
                ),
                (
                    "Tracks Dir".to_string(),
                    config.conductor.tracks_dir.clone(),
                ),
                (
                    "Auto Setup".to_string(),
                    config.conductor.auto_setup.to_string(),
                ),
                (
                    "Planning Mode".to_string(),
                    config.conductor.planning_mode.clone(),
                ),
                (
                    "Checkpoint Freq".to_string(),
                    config.conductor.checkpoint_frequency.to_string(),
                ),
            ],
            12 => vec![
                ("Log Dir".to_string(), config.paths.log_dir.clone()),
                (
                    "VS Cache Dir".to_string(),
                    config.paths.vs_cache_dir.clone(),
                ),
                (
                    "Database File".to_string(),
                    config.paths.database_file.clone(),
                ),
                (
                    "App Spec File".to_string(),
                    config.paths.app_spec_file.clone(),
                ),
                (
                    "OpenCode Paths".to_string(),
                    format_array(&config.paths.opencode_paths),
                ),
            ],
            13 => vec![
                (
                    "Enabled".to_string(),
                    config.communication.enabled.to_string(),
                ),
                (
                    "File Path".to_string(),
                    config.communication.file_path.clone(),
                ),
                (
                    "Auto Ask on Error".to_string(),
                    config.communication.auto_ask_on_error.to_string(),
                ),
                (
                    "Check Interval (sessions)".to_string(),
                    config.communication.check_interval_sessions.to_string(),
                ),
                (
                    "Max Pending Questions".to_string(),
                    config.communication.max_pending_questions.to_string(),
                ),
            ],
            _ => vec![("Placeholder".to_string(), "Value".to_string())],
        }
    };

    let fields_len = fields.len();

    // Calculate dynamic label width (max label length + padding)
    let max_label_width = fields
        .iter()
        .map(|(label, _)| label.len())
        .max()
        .unwrap_or(15)
        + 2;

    // Calculate dynamic sidebar width (max section name + indicator)
    let max_section_width = sections.iter().map(|s| s.len()).max().unwrap_or(10) + 4;

    hooks.use_terminal_events(move |event| match event {
        TerminalEvent::Key(KeyEvent { code, kind, .. }) if kind != KeyEventKind::Release => {
            if is_editing.get() {
                match code {
                    KeyCode::Enter => {
                        let mut config = config_arc.lock().unwrap();
                        let section_idx = selected_section.get();
                        let field_idx = selected_field.get();
                        let val = edit_buffer.to_string();

                        match section_idx {
                            0 => match field_idx {
                                0 => config.models.autonomous = val,
                                1 => config.models.default = val,
                                2 => config.models.reasoning = val,
                                3 => config.models.enhancement = val,
                                4 => config.models.fixer = val,
                                _ => {}
                            },
                            1 => match field_idx {
                                0 => config.generation.complexity = val,
                                1 => {
                                    config.generation.enable_subagents =
                                        val.to_lowercase() == "true"
                                }
                                2 => {
                                    config.generation.include_security_section =
                                        val.to_lowercase() == "true"
                                }
                                3 => {
                                    config.generation.include_testing_strategy =
                                        val.to_lowercase() == "true"
                                }
                                4 => {
                                    config.generation.include_devops_section =
                                        val.to_lowercase() == "true"
                                }
                                5 => {
                                    config.generation.include_accessibility =
                                        val.to_lowercase() == "true"
                                }
                                6 => {
                                    config.generation.include_future_enhancements =
                                        val.to_lowercase() == "true"
                                }
                                _ => {}
                            },
                            2 => match field_idx {
                                0 => {
                                    config.autonomous.max_iterations =
                                        val.parse().unwrap_or(config.autonomous.max_iterations)
                                }
                                1 => {
                                    config.autonomous.delay_between_sessions = val
                                        .parse()
                                        .unwrap_or(config.autonomous.delay_between_sessions)
                                }
                                2 => {
                                    config.autonomous.session_timeout_minutes = val
                                        .parse()
                                        .unwrap_or(config.autonomous.session_timeout_minutes)
                                }
                                3 => {
                                    config.autonomous.idle_timeout_seconds = val
                                        .parse()
                                        .unwrap_or(config.autonomous.idle_timeout_seconds)
                                }
                                4 => config.autonomous.auto_commit = val.to_lowercase() == "true",
                                5 => config.autonomous.log_level = val,
                                _ => {}
                            },
                            3 => match field_idx {
                                0 => {
                                    config.agent.max_retry_attempts =
                                        val.parse().unwrap_or(config.agent.max_retry_attempts)
                                }
                                1 => {
                                    config.agent.max_research_attempts =
                                        val.parse().unwrap_or(config.agent.max_research_attempts)
                                }
                                2 => {
                                    config.agent.verification_sample_size =
                                        val.parse().unwrap_or(config.agent.verification_sample_size)
                                }
                                3 => {
                                    config.agent.single_feature_focus = val.to_lowercase() == "true"
                                }
                                _ => {}
                            },
                            4 => match field_idx {
                                0 => {
                                    config.alternative_approaches.enabled =
                                        val.to_lowercase() == "true"
                                }
                                1 => {
                                    config.alternative_approaches.num_approaches = val
                                        .parse()
                                        .unwrap_or(config.alternative_approaches.num_approaches)
                                }
                                2 => {
                                    config.alternative_approaches.retry_threshold = val
                                        .parse()
                                        .unwrap_or(config.alternative_approaches.retry_threshold)
                                }
                                3 => {
                                    config.alternative_approaches.cache_results =
                                        val.to_lowercase() == "true"
                                }
                                4 => config.alternative_approaches.cache_dir = val,
                                _ => {}
                            },
                            5 => match field_idx {
                                0 => config.mcp.prefer_osgrep = val.to_lowercase() == "true",
                                1 => {
                                    config.mcp.use_sequential_thinking =
                                        val.to_lowercase() == "true"
                                }
                                idx => {
                                    if let Some(tool_name) = global_mcp_tools.read().get(idx - 2) {
                                        let enabled = val.to_lowercase() == "true";
                                        if enabled {
                                            if !config.mcp.required_tools.contains(tool_name) {
                                                config.mcp.required_tools.push(tool_name.clone());
                                            }
                                        } else {
                                            config.mcp.required_tools.retain(|t| t != tool_name);
                                        }
                                    }
                                }
                            },
                            6 => match field_idx {
                                0 => config.security.allowlist_file = val,
                                1 => {
                                    config.security.enforce_allowlist = val.to_lowercase() == "true"
                                }
                                2 => config.security.blocked_patterns = parse_array(&val),
                                _ => {}
                            },
                            7 => match field_idx {
                                0 => config.ui.colored_output = val.to_lowercase() == "true",
                                1 => config.ui.verbose = val.to_lowercase() == "true",
                                2 => config.ui.show_progress = val.to_lowercase() == "true",
                                3 => {
                                    config.ui.spec_preview_lines =
                                        val.parse().unwrap_or(config.ui.spec_preview_lines)
                                }
                                _ => {}
                            },
                            8 => match field_idx {
                                0 => {
                                    config.notifications.webhook_enabled =
                                        val.to_lowercase() == "true"
                                }
                                1 => {
                                    config.notifications.webhook_url =
                                        if val.is_empty() { None } else { Some(val) }
                                }
                                _ => {}
                            },
                            9 => match field_idx {
                                0 => config.features.categories = parse_array(&val),
                                1 => config.features.priorities = parse_array(&val),
                                2 => {
                                    config.features.require_verification_command =
                                        val.to_lowercase() == "true"
                                }
                                3 => {
                                    config.features.narrow_test_min_steps =
                                        val.parse().unwrap_or(config.features.narrow_test_min_steps)
                                }
                                4 => {
                                    config.features.narrow_test_max_steps =
                                        val.parse().unwrap_or(config.features.narrow_test_max_steps)
                                }
                                5 => {
                                    config.features.comprehensive_test_min_steps = val
                                        .parse()
                                        .unwrap_or(config.features.comprehensive_test_min_steps)
                                }
                                _ => {}
                            },
                            10 => match field_idx {
                                0 => config.scaffolding.git_init = val.to_lowercase() == "true",
                                1 => config.scaffolding.output_dir = val,
                                2 => {
                                    config.scaffolding.create_opencode_dir =
                                        val.to_lowercase() == "true"
                                }
                                3 => {
                                    config.scaffolding.create_scripts_dir =
                                        val.to_lowercase() == "true"
                                }
                                _ => {}
                            },
                            11 => match field_idx {
                                0 => config.conductor.context_dir = val,
                                1 => config.conductor.tracks_dir = val,
                                2 => config.conductor.auto_setup = val.to_lowercase() == "true",
                                3 => config.conductor.planning_mode = val,
                                4 => {
                                    config.conductor.checkpoint_frequency =
                                        val.parse().unwrap_or(config.conductor.checkpoint_frequency)
                                }
                                _ => {}
                            },
                            12 => match field_idx {
                                0 => config.paths.log_dir = val,
                                1 => config.paths.vs_cache_dir = val,
                                2 => config.paths.database_file = val,
                                3 => config.paths.app_spec_file = val,
                                4 => config.paths.opencode_paths = parse_array(&val),
                                _ => {}
                            },
                            13 => match field_idx {
                                0 => config.communication.enabled = val.to_lowercase() == "true",
                                1 => config.communication.file_path = val,
                                2 => {
                                    config.communication.auto_ask_on_error =
                                        val.to_lowercase() == "true"
                                }
                                3 => {
                                    config.communication.check_interval_sessions = val
                                        .parse()
                                        .unwrap_or(config.communication.check_interval_sessions)
                                }
                                4 => {
                                    config.communication.max_pending_questions = val
                                        .parse()
                                        .unwrap_or(config.communication.max_pending_questions)
                                }
                                _ => {}
                            },
                            _ => {}
                        }
                        is_editing.set(false);
                    }
                    KeyCode::Esc => {
                        is_editing.set(false);
                    }
                    KeyCode::Backspace => {
                        let mut b = edit_buffer.to_string();
                        b.pop();
                        edit_buffer.set(b);
                    }
                    KeyCode::Char(c) => {
                        let mut b = edit_buffer.to_string();
                        b.push(c);
                        edit_buffer.set(b);
                    }
                    _ => {}
                }
            } else {
                match code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        if selected_field.get() > 0 {
                            selected_field.set(selected_field.get() - 1);
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if selected_field.get() < fields_len - 1 {
                            selected_field.set(selected_field.get() + 1);
                        }
                    }
                    KeyCode::Left | KeyCode::Char('h') => {
                        if selected_section.get() > 0 {
                            selected_section.set(selected_section.get() - 1);
                            selected_field.set(0);
                        }
                    }
                    KeyCode::Right | KeyCode::Char('l') => {
                        if selected_section.get() < sections_len - 1 {
                            selected_section.set(selected_section.get() + 1);
                            selected_field.set(0);
                        }
                    }
                    KeyCode::Enter => {
                        let config = config_arc.lock().unwrap();
                        let section_idx = selected_section.get();
                        let field_idx = selected_field.get();
                        let val = match section_idx {
                            0 => match field_idx {
                                0 => config.models.autonomous.clone(),
                                1 => config.models.default.clone(),
                                2 => config.models.reasoning.clone(),
                                3 => config.models.enhancement.clone(),
                                4 => config.models.fixer.clone(),
                                _ => String::new(),
                            },
                            1 => match field_idx {
                                0 => config.generation.complexity.clone(),
                                1 => config.generation.enable_subagents.to_string(),
                                2 => config.generation.include_security_section.to_string(),
                                3 => config.generation.include_testing_strategy.to_string(),
                                4 => config.generation.include_devops_section.to_string(),
                                5 => config.generation.include_accessibility.to_string(),
                                6 => config.generation.include_future_enhancements.to_string(),
                                _ => String::new(),
                            },
                            2 => match field_idx {
                                0 => config.autonomous.max_iterations.to_string(),
                                1 => config.autonomous.delay_between_sessions.to_string(),
                                2 => config.autonomous.session_timeout_minutes.to_string(),
                                3 => config.autonomous.idle_timeout_seconds.to_string(),
                                4 => config.autonomous.auto_commit.to_string(),
                                5 => config.autonomous.log_level.clone(),
                                _ => String::new(),
                            },
                            3 => match field_idx {
                                0 => config.agent.max_retry_attempts.to_string(),
                                1 => config.agent.max_research_attempts.to_string(),
                                2 => config.agent.verification_sample_size.to_string(),
                                3 => config.agent.single_feature_focus.to_string(),
                                _ => String::new(),
                            },
                            4 => match field_idx {
                                0 => config.alternative_approaches.enabled.to_string(),
                                1 => config.alternative_approaches.num_approaches.to_string(),
                                2 => config.alternative_approaches.retry_threshold.to_string(),
                                3 => config.alternative_approaches.cache_results.to_string(),
                                4 => config.alternative_approaches.cache_dir.clone(),
                                _ => String::new(),
                            },
                            5 => match field_idx {
                                0 => config.mcp.prefer_osgrep.to_string(),
                                1 => config.mcp.use_sequential_thinking.to_string(),
                                idx => {
                                    if let Some(tool_name) = global_mcp_tools.read().get(idx - 2) {
                                        config.mcp.required_tools.contains(tool_name).to_string()
                                    } else {
                                        String::new()
                                    }
                                }
                            },
                            6 => match field_idx {
                                0 => config.security.allowlist_file.clone(),
                                1 => config.security.enforce_allowlist.to_string(),
                                2 => format_array(&config.security.blocked_patterns),
                                _ => String::new(),
                            },
                            7 => match field_idx {
                                0 => config.ui.colored_output.to_string(),
                                1 => config.ui.verbose.to_string(),
                                2 => config.ui.show_progress.to_string(),
                                3 => config.ui.spec_preview_lines.to_string(),
                                _ => String::new(),
                            },
                            8 => match field_idx {
                                0 => config.notifications.webhook_enabled.to_string(),
                                1 => config.notifications.webhook_url.clone().unwrap_or_default(),
                                _ => String::new(),
                            },
                            9 => match field_idx {
                                0 => format_array(&config.features.categories),
                                1 => format_array(&config.features.priorities),
                                2 => config.features.require_verification_command.to_string(),
                                3 => config.features.narrow_test_min_steps.to_string(),
                                4 => config.features.narrow_test_max_steps.to_string(),
                                5 => config.features.comprehensive_test_min_steps.to_string(),
                                _ => String::new(),
                            },
                            10 => match field_idx {
                                0 => config.scaffolding.git_init.to_string(),
                                1 => config.scaffolding.output_dir.clone(),
                                2 => config.scaffolding.create_opencode_dir.to_string(),
                                3 => config.scaffolding.create_scripts_dir.to_string(),
                                _ => String::new(),
                            },
                            11 => match field_idx {
                                0 => config.conductor.context_dir.clone(),
                                1 => config.conductor.tracks_dir.clone(),
                                2 => config.conductor.auto_setup.to_string(),
                                3 => config.conductor.planning_mode.clone(),
                                4 => config.conductor.checkpoint_frequency.to_string(),
                                _ => String::new(),
                            },
                            12 => match field_idx {
                                0 => config.paths.log_dir.clone(),
                                1 => config.paths.vs_cache_dir.clone(),
                                2 => config.paths.database_file.clone(),
                                3 => config.paths.app_spec_file.clone(),
                                4 => format_array(&config.paths.opencode_paths),
                                _ => String::new(),
                            },
                            13 => match field_idx {
                                0 => config.communication.enabled.to_string(),
                                1 => config.communication.file_path.clone(),
                                2 => config.communication.auto_ask_on_error.to_string(),
                                3 => config.communication.check_interval_sessions.to_string(),
                                4 => config.communication.max_pending_questions.to_string(),
                                _ => String::new(),
                            },
                            _ => String::new(),
                        };
                        edit_buffer.set(val);
                        is_editing.set(true);
                    }
                    KeyCode::Char('q') => {
                        should_exit.set(true);
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    });

    if should_exit.get() {
        system.exit();
    }

    element! {
        View(width, height, flex_direction: FlexDirection::Column) {
            // Header - Clean Minimalist
            View(padding: 1, border_style: BorderStyle::Round, border_color: Color::DarkGrey) {
                Text(content: "⚙  Configuration", weight: Weight::Bold, color: Color::White)
            }

            View(flex_grow: 1.0, flex_direction: FlexDirection::Row) {
                // Sidebar: Sections
                View(width: max_section_width as u32, flex_direction: FlexDirection::Column, border_style: BorderStyle::Single, border_color: Color::DarkGrey, padding_top: 1) {
                    #(sections.iter().enumerate().map(|(i, &name)| {
                        let is_selected = i == selected_section.get();
                        element! {
                            View(padding_left: 1) {
                                Text(
                                    content: if is_selected { format!("› {}", name) } else { format!("  {}", name) },
                                    color: if is_selected { Color::White } else { Color::Grey },
                                    weight: if is_selected { Weight::Bold } else { Weight::Normal },
                                )
                            }
                        }
                    }))
                }

                // Main: Fields
                View(flex_grow: 1.0, flex_direction: FlexDirection::Column, padding: 1, border_style: BorderStyle::Single, border_color: Color::DarkGrey) {
                    Text(content: sections[selected_section.get()].to_string(), weight: Weight::Bold, color: Color::White)

                   // Fields List
                View(flex_direction: FlexDirection::Column, flex_grow: 1.0, margin_top: 1) {
                    #(fields.iter().enumerate().map(|(i, (label, value)): (usize, &(String, String))| {
                        let label = label.to_string();
                        let value_str = value.to_string();
                        let is_selected = i == selected_field.get();
                        let is_on_editing = is_selected && is_editing.get();

                        // Detect if this is a boolean value
                        let is_boolean = value_str == "true" || value_str == "false";
                        let bool_val = value_str == "true";

                        element! (
                            View(margin_bottom: 0) {
                                Text(
                                    content: if is_selected { "› " } else { "  " },
                                    color: Color::Cyan,
                                )
                                View(width: max_label_width as u32) {
                                    Text(
                                        content: label.clone(),
                                        color: if is_selected { Color::White } else { Color::Grey },
                                    )
                                }
                                #(if is_boolean {
                                    // Toggle switch for booleans
                                    element! {
                                        View(flex_direction: FlexDirection::Row) {
                                            Text(
                                                content: if bool_val { "[ON] " } else { " ON  " },
                                                color: if bool_val { Color::Cyan } else { Color::DarkGrey },
                                                weight: if bool_val { Weight::Bold } else { Weight::Normal },
                                            )
                                            Text(
                                                content: if !bool_val { "[OFF]" } else { " OFF " },
                                                color: if !bool_val { Color::Cyan } else { Color::DarkGrey },
                                                weight: if !bool_val { Weight::Bold } else { Weight::Normal },
                                            )
                                        }
                                    }
                                } else if is_on_editing {
                                    // Editing mode with cursor
                                    element! {
                                        View(border_style: BorderStyle::Single, border_color: Color::Cyan, padding_left: 1, padding_right: 1) {
                                            Text(
                                                content: format!("{}▏", edit_buffer.read().as_str()),
                                                color: Color::White,
                                            )
                                        }
                                    }
                                } else {
                                    // Normal display with box
                                    element! {
                                        View(border_style: BorderStyle::Single, border_color: if is_selected { Color::Grey } else { Color::DarkGrey }, padding_left: 1, padding_right: 1) {
                                            Text(
                                                content: value_str.clone(),
                                                color: if is_selected { Color::White } else { Color::DarkGrey },
                                            )
                                        }
                                    }
                                })
                            }
                        )
                    }))
                }

                    View(margin_top: 2, padding: 1, border_style: BorderStyle::Single, border_color: Color::DarkGrey) {
                        Text(content: "Note: Press Enter to edit the selected field. Boolean values should be typed as 'true' or 'false'.", color: Color::Grey)
                    }
                }
            }

            // Footer
            View(padding: 1, border_style: BorderStyle::Single, border_color: Color::DarkGrey) {
                Text(content: "↑↓ Navigate  ←→ Sections  Enter Edit  q Save & Quit", color: Color::DarkGrey)
            }
        }
    }
}

/// Run the fullscreen configuration editor
pub fn run_fullscreen_config_editor(
    config: &mut Config,
    available_models: Vec<String>,
) -> Result<()> {
    let shared_config = Arc::new(Mutex::new(config.clone()));
    let shared_config_clone = shared_config.clone();

    smol::block_on(async {
        element! {
            ConfigEditor(config: shared_config_clone, available_models: available_models)
        }
        .fullscreen()
        .await
    })?;

    *config = shared_config.lock().unwrap().clone();
    Ok(())
}
