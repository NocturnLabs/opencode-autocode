//! Fullscreen interactive setup TUI using iocraft
//!
//! Provides a modern fullscreen terminal interface for the interactive setup flow.

use anyhow::Result;
use iocraft::prelude::*;
use std::path::Path;

use crate::config::Config;
use crate::scaffold::{scaffold_custom, scaffold_default};
use crate::tui::validation::SpecAction;
use crate::validation::ValidationResult;
use std::sync::{Arc, Mutex};

/// Interactive mode options
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum InteractiveMode {
    #[default]
    Generated,
    Manual,
    FromSpecFile,
    Default,
}

impl InteractiveMode {
    fn all() -> &'static [InteractiveMode] {
        &[
            InteractiveMode::Generated,
            InteractiveMode::Manual,
            InteractiveMode::FromSpecFile,
            InteractiveMode::Default,
        ]
    }

    fn label(&self) -> &'static str {
        match self {
            InteractiveMode::Generated => {
                "🤖 AI Generated - Let AI research and create a full spec"
            }
            InteractiveMode::Manual => "📝 Manual - Fill out project details step by step",
            InteractiveMode::FromSpecFile => "📁 From File - Use an existing app_spec.md",
            InteractiveMode::Default => "⚡ Default - Use built-in specification",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            InteractiveMode::Generated => {
                "AI will analyze your project idea and generate a comprehensive specification"
            }
            InteractiveMode::Manual => {
                "Walk through a form to define project name, features, and tech stack"
            }
            InteractiveMode::FromSpecFile => "Load an existing spec file from disk",
            InteractiveMode::Default => "Use a minimal built-in template to get started quickly",
        }
    }
}

/// Application phase for the fullscreen TUI
#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum Phase {
    #[default]
    SetupChoice,
    ModeSelection,
    Done,
}

#[derive(Props, Default)]
struct InteractiveSetupProps {
    has_existing_config: bool,
    result: Arc<Mutex<Option<SetupResult>>>,
}

#[component]
fn InteractiveSetup(
    props: &InteractiveSetupProps,
    mut hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let (width, height) = hooks.use_terminal_size();
    let mut system = hooks.use_context_mut::<SystemContext>();

    let mut selected_mode = hooks.use_state(|| 0usize);
    let mut setup_choice = hooks.use_state(|| 0usize);
    let show_reconfigure = hooks.use_state(|| props.has_existing_config);
    let mut reconfigure_selected = hooks.use_state(|| false);
    let mut phase = hooks.use_state(|| Phase::SetupChoice);
    let mut should_exit = hooks.use_state(|| false);
    let mut result_mode = hooks.use_state(|| None::<usize>);

    let modes = InteractiveMode::all();

    hooks.use_terminal_events(move |event| match event {
        TerminalEvent::Key(KeyEvent { code, kind, .. }) if kind != KeyEventKind::Release => {
            match phase.get() {
                Phase::SetupChoice => {
                    if show_reconfigure.get() {
                        // Yes/No for reconfigure
                        match code {
                            KeyCode::Left
                            | KeyCode::Right
                            | KeyCode::Char('h')
                            | KeyCode::Char('l') => {
                                reconfigure_selected.set(!reconfigure_selected.get());
                            }
                            KeyCode::Char('y') | KeyCode::Char('Y') => {
                                reconfigure_selected.set(true);
                                phase.set(Phase::ModeSelection);
                            }
                            KeyCode::Char('n') | KeyCode::Char('N') => {
                                reconfigure_selected.set(false);
                                phase.set(Phase::ModeSelection);
                            }
                            KeyCode::Enter => {
                                phase.set(Phase::ModeSelection);
                            }
                            KeyCode::Esc | KeyCode::Char('q') => {
                                should_exit.set(true);
                            }
                            _ => {}
                        }
                    } else {
                        // Quick start vs Configure
                        match code {
                            KeyCode::Up | KeyCode::Char('k') => {
                                if setup_choice.get() > 0 {
                                    setup_choice.set(setup_choice.get() - 1);
                                }
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                if setup_choice.get() < 1 {
                                    setup_choice.set(setup_choice.get() + 1);
                                }
                            }
                            KeyCode::Enter => {
                                phase.set(Phase::ModeSelection);
                            }
                            KeyCode::Esc | KeyCode::Char('q') => {
                                should_exit.set(true);
                            }
                            _ => {}
                        }
                    }
                }
                Phase::ModeSelection => match code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        if selected_mode.get() > 0 {
                            selected_mode.set(selected_mode.get() - 1);
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if selected_mode.get() < modes.len() - 1 {
                            selected_mode.set(selected_mode.get() + 1);
                        }
                    }
                    KeyCode::Enter => {
                        result_mode.set(Some(selected_mode.get()));
                        phase.set(Phase::Done);
                    }
                    KeyCode::Esc | KeyCode::Char('q') => {
                        should_exit.set(true);
                    }
                    _ => {}
                },
                Phase::Done => {}
            }
        }
        _ => {}
    });

    if should_exit.get() || phase.get() == Phase::Done {
        if phase.get() == Phase::Done {
            let mut res = props.result.lock().unwrap();
            *res = Some(SetupResult {
                mode: Some(modes[selected_mode.get()]),
                should_configure: !props.has_existing_config && setup_choice.get() == 1,
                reconfigure: props.has_existing_config && reconfigure_selected.get(),
            });
        }
        system.exit();
    }

    let current_mode = modes.get(selected_mode.get()).copied().unwrap_or_default();

    element! {
        View(
            width,
            height,
            flex_direction: FlexDirection::Column,
        ) {
            // Header
            View(
                border_style: BorderStyle::Round,
                border_color: Color::Blue,
                padding: 1,
                margin_bottom: 1,
                align_items: AlignItems::Center,
            ) {
                Text(
                    content: "✨ OpenCode Forger - Interactive Setup",
                    weight: Weight::Bold,
                    color: Color::Cyan,
                )
            }

            // Main content area
            View(
                flex_grow: 1.0,
                padding: 1,
                flex_direction: FlexDirection::Column,
            ) {
                #(match phase.get() {
                    Phase::SetupChoice => {
                        if show_reconfigure.get() {
                            element! {
                                View(flex_direction: FlexDirection::Column) {
                                    Text(
                                        content: "Found existing configuration.",
                                        color: Color::Yellow,
                                        weight: Weight::Bold,
                                    )
                                    View(margin_top: 1) {
                                        Text(content: "Reconfigure settings? ", color: Color::White)
                                        Text(
                                            content: "[Yes]",
                                            color: if reconfigure_selected.get() { Color::Green } else { Color::Grey },
                                            weight: if reconfigure_selected.get() { Weight::Bold } else { Weight::Normal },
                                        )
                                        Text(content: " / ")
                                        Text(
                                            content: "[No]",
                                            color: if !reconfigure_selected.get() { Color::Green } else { Color::Grey },
                                            weight: if !reconfigure_selected.get() { Weight::Bold } else { Weight::Normal },
                                        )
                                    }
                                    Text(
                                        content: "(←/→ to toggle, Enter to confirm)",
                                        color: Color::Grey,
                                    )
                                }
                            }
                        } else {
                            element! {
                                View(flex_direction: FlexDirection::Column) {
                                    Text(
                                        content: "Setup Mode",
                                        weight: Weight::Bold,
                                        color: Color::Green,
                                    )
                                    View(margin_top: 1, flex_direction: FlexDirection::Column) {
                                        View {
                                            Text(
                                                content: if setup_choice.get() == 0 { "▸ " } else { "  " },
                                                color: Color::Green,
                                            )
                                            Text(
                                                content: "⚡ Quick start (use defaults)",
                                                color: if setup_choice.get() == 0 { Color::Green } else { Color::White },
                                                weight: if setup_choice.get() == 0 { Weight::Bold } else { Weight::Normal },
                                            )
                                        }
                                        View {
                                            Text(
                                                content: if setup_choice.get() == 1 { "▸ " } else { "  " },
                                                color: Color::Green,
                                            )
                                            Text(
                                                content: "⚙️  Configure settings first",
                                                color: if setup_choice.get() == 1 { Color::Green } else { Color::White },
                                                weight: if setup_choice.get() == 1 { Weight::Bold } else { Weight::Normal },
                                            )
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Phase::ModeSelection => {
                        element! {
                            View(flex_direction: FlexDirection::Column) {
                                Text(
                                    content: "How would you like to create your project spec?",
                                    weight: Weight::Bold,
                                    color: Color::Green,
                                )
                                View(margin_top: 1, flex_direction: FlexDirection::Column) {
                                    #(modes.iter().enumerate().map(|(i, mode)| {
                                        let is_selected = i == selected_mode.get();
                                        element! {
                                            View {
                                                Text(
                                                    content: if is_selected { "▸ " } else { "  " },
                                                    color: Color::Green,
                                                )
                                                Text(
                                                    content: mode.label(),
                                                    color: if is_selected { Color::Green } else { Color::White },
                                                    weight: if is_selected { Weight::Bold } else { Weight::Normal },
                                                )
                                            }
                                        }
                                    }))
                                }
                                // Description of selected mode
                                View(
                                    margin_top: 2,
                                    padding: 1,
                                    border_style: BorderStyle::Single,
                                    border_color: Color::Grey,
                                ) {
                                    Text(
                                        content: current_mode.description(),
                                        color: Color::Grey,
                                    )
                                }
                            }
                        }
                    }
                    Phase::Done => {
                        element! {
                            View {
                                Text(content: "Starting...", color: Color::Green)
                            }
                        }
                    }
                })
            }

            // Footer
            View(
                border_style: BorderStyle::Single,
                border_color: Color::Grey,
                padding: 1,
            ) {
                Text(
                    content: "↑/↓: Navigate  Enter: Select  q: Quit",
                    color: Color::Grey,
                )
            }
        }
    }
}

/// Result from the fullscreen TUI
pub struct SetupResult {
    pub mode: Option<InteractiveMode>,
    pub should_configure: bool,
    pub reconfigure: bool,
}

/// Run the fullscreen interactive setup and return the selected mode
pub fn run_fullscreen_setup(has_existing_config: bool) -> Result<SetupResult> {
    let result = Arc::new(Mutex::new(None));
    let result_clone = result.clone();

    smol::block_on(async {
        element! {
            InteractiveSetup(has_existing_config: has_existing_config, result: result_clone)
        }
        .fullscreen()
        .await
    })?;

    let res = result.lock().unwrap().take();
    Ok(res.unwrap_or(SetupResult {
        mode: None,
        should_configure: false,
        reconfigure: false,
    }))
}

#[derive(Props, Default)]
struct SpecReviewProps {
    spec_text: String,
    validation: ValidationResult,
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
                // Left side: Validation & Actions
                View(width: 40, flex_direction: FlexDirection::Column, padding: 1, border_style: BorderStyle::Single, border_color: Color::Grey) {
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

                // Right side: Preview
                View(flex_grow: 1.0, flex_direction: FlexDirection::Column, padding: 1, border_style: BorderStyle::Single, border_color: Color::Grey) {
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

pub fn run_interactive(output_dir: &Path, use_subagents: bool) -> Result<()> {
    let config_path = output_dir.join(".forger/config.toml");
    let has_existing_config = config_path.exists();

    // Run fullscreen TUI for mode selection
    let result = run_fullscreen_setup(has_existing_config)?;

    if result.mode.is_none() {
        println!("Cancelled.");
        return Ok(());
    }

    // Handle configuration
    let config = if result.reconfigure || result.should_configure {
        crate::config_tui::run_config_tui(Some(output_dir))?
    } else {
        Config::load(Some(output_dir)).unwrap_or_default()
    };

    // Execute the selected mode
    match result.mode {
        Some(InteractiveMode::Generated) => {
            super::generated::run_generated_mode(output_dir, &config, use_subagents)
        }
        Some(InteractiveMode::Manual) => super::manual::run_manual_mode(output_dir),
        Some(InteractiveMode::FromSpecFile) => run_from_spec_file_mode(output_dir),
        Some(InteractiveMode::Default) => run_default_mode(output_dir),
        None => unreachable!(),
    }
}

fn run_from_spec_file_mode(output_dir: &Path) -> Result<()> {
    use super::prompts::input;
    println!("\n📁 Spec File Mode");
    let spec_path = input("Path to spec file", Some("app_spec.md"))?;

    let spec_path = std::path::PathBuf::from(&spec_path);
    if !spec_path.exists() {
        println!("❌ Spec file not found: {}", spec_path.display());
        return Ok(());
    }

    scaffold_custom(output_dir, &spec_path)?;
    println!("\n✅ Project scaffolded from spec file!");
    Ok(())
}

fn run_default_mode(output_dir: &Path) -> Result<()> {
    use super::prompts::confirm;

    println!("\n⚡ Default Mode");
    println!("Using the built-in default specification.");

    if confirm("Scaffold project with default spec?", true)? {
        scaffold_default(output_dir)?;
        println!("\n✅ Project scaffolded with default spec!");
    } else {
        println!("❌ Cancelled.");
    }
    Ok(())
}
#[derive(Props, Default)]
struct ConfigEditorProps {
    config: Arc<Mutex<Config>>,
    available_models: Vec<String>,
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
    let sections = vec![
        "Models",
        "Autonomous",
        "Agent",
        "Recovery",
        "MCP",
        "Generation",
        "Security",
        "Communication",
        "UI",
        "Features",
        "Scaffolding",
        "Notifications",
        "Conductor",
        "Paths",
    ];
    let sections_len = sections.len();
    let _sections_for_events = sections.clone();

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
                    "Auto Commit".to_string(),
                    config.autonomous.auto_commit.to_string(),
                ),
                ("Log Level".to_string(), config.autonomous.log_level.clone()),
            ],
            2 => vec![
                (
                    "Max Retries".to_string(),
                    config.agent.max_retry_attempts.to_string(),
                ),
                (
                    "Max Research".to_string(),
                    config.agent.max_research_attempts.to_string(),
                ),
                (
                    "Single Focus".to_string(),
                    config.agent.single_feature_focus.to_string(),
                ),
            ],
            3 => vec![
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
            4 => {
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
            5 => vec![
                (
                    "Complexity".to_string(),
                    config.generation.complexity.clone(),
                ),
                (
                    "Accessibility".to_string(),
                    config.generation.include_accessibility.to_string(),
                ),
                (
                    "Security Sec".to_string(),
                    config.generation.include_security_section.to_string(),
                ),
                (
                    "Testing Sec".to_string(),
                    config.generation.include_testing_strategy.to_string(),
                ),
            ],
            6 => vec![
                (
                    "Allowlist File".to_string(),
                    config.security.allowlist_file.clone(),
                ),
                (
                    "Strict".to_string(),
                    config.security.enforce_allowlist.to_string(),
                ),
            ],
            7 => vec![
                (
                    "Enabled".to_string(),
                    config.communication.enabled.to_string(),
                ),
                ("Path".to_string(), config.communication.file_path.clone()),
                (
                    "Auto Ask".to_string(),
                    config.communication.auto_ask_on_error.to_string(),
                ),
                (
                    "Check Interval".to_string(),
                    config.communication.check_interval_sessions.to_string(),
                ),
                (
                    "Max Pending".to_string(),
                    config.communication.max_pending_questions.to_string(),
                ),
            ],
            8 => vec![
                (
                    "Spec Preview Lines".to_string(),
                    config.ui.spec_preview_lines.to_string(),
                ),
                (
                    "Colored Output".to_string(),
                    config.ui.colored_output.to_string(),
                ),
                (
                    "Show Progress".to_string(),
                    config.ui.show_progress.to_string(),
                ),
                ("Verbose".to_string(), config.ui.verbose.to_string()),
            ],
            9 => vec![
                (
                    "Require Verification".to_string(),
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
                    "Comprehensive Min".to_string(),
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
                    "Create .opencode".to_string(),
                    config.scaffolding.create_opencode_dir.to_string(),
                ),
                (
                    "Create Scripts".to_string(),
                    config.scaffolding.create_scripts_dir.to_string(),
                ),
            ],
            11 => vec![
                (
                    "Webhook Enabled".to_string(),
                    config.notifications.webhook_enabled.to_string(),
                ),
                (
                    "Webhook URL".to_string(),
                    config.notifications.webhook_url.clone().unwrap_or_default(),
                ),
            ],
            12 => vec![
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
            13 => vec![
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
            ],
            _ => vec![("Placeholder".to_string(), "Value".to_string())],
        }
    };

    let fields_len = fields.len();

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
                                3 => config.autonomous.auto_commit = val.to_lowercase() == "true",
                                4 => config.autonomous.log_level = val,
                                _ => {}
                            },
                            2 => match field_idx {
                                0 => {
                                    config.agent.max_retry_attempts =
                                        val.parse().unwrap_or(config.agent.max_retry_attempts)
                                }
                                1 => {
                                    config.agent.max_research_attempts =
                                        val.parse().unwrap_or(config.agent.max_research_attempts)
                                }
                                2 => {
                                    config.agent.single_feature_focus = val.to_lowercase() == "true"
                                }
                                _ => {}
                            },
                            3 => match field_idx {
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
                            4 => match field_idx {
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
                            5 => match field_idx {
                                0 => config.generation.complexity = val,
                                1 => {
                                    config.generation.include_accessibility =
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
                                _ => {}
                            },
                            6 => match field_idx {
                                0 => config.security.allowlist_file = val,
                                1 => {
                                    config.security.enforce_allowlist = val.to_lowercase() == "true"
                                }
                                _ => {}
                            },
                            7 => match field_idx {
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
                            8 => match field_idx {
                                0 => {
                                    config.ui.spec_preview_lines =
                                        val.parse().unwrap_or(config.ui.spec_preview_lines)
                                }
                                1 => config.ui.colored_output = val.to_lowercase() == "true",
                                2 => config.ui.show_progress = val.to_lowercase() == "true",
                                3 => config.ui.verbose = val.to_lowercase() == "true",
                                _ => {}
                            },
                            9 => match field_idx {
                                0 => {
                                    config.features.require_verification_command =
                                        val.to_lowercase() == "true"
                                }
                                1 => {
                                    config.features.narrow_test_min_steps =
                                        val.parse().unwrap_or(config.features.narrow_test_min_steps)
                                }
                                2 => {
                                    config.features.narrow_test_max_steps =
                                        val.parse().unwrap_or(config.features.narrow_test_max_steps)
                                }
                                3 => {
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
                            12 => match field_idx {
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
                            13 => match field_idx {
                                0 => config.paths.log_dir = val,
                                1 => config.paths.vs_cache_dir = val,
                                2 => config.paths.database_file = val,
                                3 => config.paths.app_spec_file = val,
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
                                0 => config.autonomous.max_iterations.to_string(),
                                1 => config.autonomous.delay_between_sessions.to_string(),
                                2 => config.autonomous.session_timeout_minutes.to_string(),
                                3 => config.autonomous.auto_commit.to_string(),
                                4 => config.autonomous.log_level.clone(),
                                _ => String::new(),
                            },
                            2 => match field_idx {
                                0 => config.agent.max_retry_attempts.to_string(),
                                1 => config.agent.max_research_attempts.to_string(),
                                2 => config.agent.single_feature_focus.to_string(),
                                _ => String::new(),
                            },
                            3 => match field_idx {
                                0 => config.alternative_approaches.enabled.to_string(),
                                1 => config.alternative_approaches.num_approaches.to_string(),
                                2 => config.alternative_approaches.retry_threshold.to_string(),
                                _ => String::new(),
                            },
                            4 => match field_idx {
                                0 => config.mcp.prefer_osgrep.to_string(),
                                1 => config.mcp.use_sequential_thinking.to_string(),
                                _ => String::new(),
                            },
                            5 => match field_idx {
                                0 => config.generation.complexity.clone(),
                                1 => config.generation.include_accessibility.to_string(),
                                2 => config.generation.include_security_section.to_string(),
                                3 => config.generation.include_testing_strategy.to_string(),
                                _ => String::new(),
                            },
                            6 => match field_idx {
                                0 => config.security.allowlist_file.clone(),
                                1 => config.security.enforce_allowlist.to_string(),
                                _ => String::new(),
                            },
                            7 => match field_idx {
                                0 => config.communication.enabled.to_string(),
                                1 => config.communication.file_path.clone(),
                                2 => config.communication.auto_ask_on_error.to_string(),
                                _ => String::new(),
                            },
                            8 => match field_idx {
                                0 => config.ui.spec_preview_lines.to_string(),
                                1 => config.ui.colored_output.to_string(),
                                2 => config.ui.show_progress.to_string(),
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
            // Header
            View(padding: 1, border_style: BorderStyle::Round, border_color: Color::Blue) {
                Text(content: "⚙️  Configuration Editor", weight: Weight::Bold, color: Color::Blue)
            }

            View(flex_grow: 1.0, flex_direction: FlexDirection::Row) {
                // Sidebar: Sections
                View(width: 20, flex_direction: FlexDirection::Column, border_style: BorderStyle::Single, border_color: Color::Grey) {
                    #(sections.iter().enumerate().map(|(i, &name)| {
                        let is_selected = i == selected_section.get();
                        element! {
                            View(padding_left: 1) {
                                Text(
                                    content: if is_selected { format!("▸ {}", name) } else { format!("  {}", name) },
                                    color: if is_selected { Color::Green } else { Color::White },
                                    weight: if is_selected { Weight::Bold } else { Weight::Normal },
                                )
                            }
                        }
                    }))
                }

                // Main: Fields
                View(flex_grow: 1.0, flex_direction: FlexDirection::Column, padding: 1, border_style: BorderStyle::Single, border_color: Color::Grey) {
                    Text(content: format!("Section: {}", sections[selected_section.get()]), weight: Weight::Bold, color: Color::Cyan)

                   // Fields List
                View(flex_direction: FlexDirection::Column, flex_grow: 1.0) {
                    #(fields.iter().enumerate().map(|(i, (label, value)): (usize, &(String, String))| {
                        let label = label.to_string();
                        let value = value.to_string();
                        let is_selected = i == selected_field.get();
                        let is_on_editing = is_selected && is_editing.get();

                        element! (
                            View {
                                Text(
                                    content: if is_selected { "▸ " } else { "  " },
                                    color: Color::Green,
                                )
                                View(width: 25) {
                                    Text(content: format!("{}: ", label))
                                }
                                Text(
                                    content: if is_on_editing { format!("{}▮", edit_buffer.read().as_str()) } else { value.clone() },
                                    color: if is_on_editing { Color::Yellow } else if is_selected { Color::White } else { Color::Grey },
                                    weight: if is_selected { Weight::Bold } else { Weight::Normal },
                                )
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
            View(padding: 1, border_style: BorderStyle::Single, border_color: Color::Grey) {
                Text(content: "Arrows/hjkl: Navigate  Enter: Edit  Esc: Back  q: Save & Quit", color: Color::Grey)
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
