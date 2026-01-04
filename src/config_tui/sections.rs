//! Section handlers for each configuration category

use anyhow::Result;
use dialoguer::{Confirm, Input};

use super::helpers::{display_hint, display_section};
use super::models::{parse_comma_list, prompt_list_selection, prompt_model_selection};
use crate::config::Config;

/// Configure AI model selections
pub fn configure_models(config: &mut Config, available_models: &[String]) -> Result<()> {
    display_section(
        "Models",
        "Configure which AI models to use for different tasks",
    );

    display_hint("Model used for the autonomous vibe loop coding sessions");
    config.models.autonomous = prompt_model_selection(
        "Autonomous model",
        available_models,
        &config.models.autonomous,
    )?;

    display_hint("Model used for generating initial app specifications");
    config.models.default = prompt_model_selection(
        "Default/Spec model",
        available_models,
        &config.models.default,
    )?;

    display_hint("Model used for reasoning and planning complex decisions");
    config.models.reasoning = prompt_model_selection(
        "Reasoning model",
        available_models,
        &config.models.reasoning,
    )?;

    display_hint("Model used for discovering enhancement opportunities");
    config.models.enhancement = prompt_model_selection(
        "Enhancement model",
        available_models,
        &config.models.enhancement,
    )?;

    display_hint("Model used for fixing malformed XML during spec generation retries");
    config.models.fixer =
        prompt_model_selection("Fixer model", available_models, &config.models.fixer)?;

    Ok(())
}

/// Configure autonomous loop behavior
pub fn configure_autonomous(config: &mut Config) -> Result<()> {
    display_section("Autonomous Loop", "Control how the vibe loop runs");

    display_hint("Maximum iterations (0 = run forever until all features pass)");
    config.autonomous.max_iterations = Input::new()
        .with_prompt("Max iterations")
        .default(config.autonomous.max_iterations)
        .interact()?;

    display_hint("Seconds between iterations (prevents rate limiting)");
    config.autonomous.delay_between_sessions = Input::new()
        .with_prompt("Delay between sessions (seconds)")
        .default(config.autonomous.delay_between_sessions)
        .interact()?;

    display_hint("Maximum session time in minutes (0 = no timeout)");
    config.autonomous.session_timeout_minutes = Input::new()
        .with_prompt("Session timeout (minutes)")
        .default(config.autonomous.session_timeout_minutes)
        .interact()?;

    display_hint("Automatically commit when a feature is completed");
    config.autonomous.auto_commit = Confirm::new()
        .with_prompt("Auto-commit on feature completion?")
        .default(config.autonomous.auto_commit)
        .interact()?;

    display_hint("Logging verbosity: DEBUG, INFO, WARN, ERROR");
    config.autonomous.log_level = prompt_list_selection(
        "Log level",
        &["DEBUG", "INFO", "WARN", "ERROR"],
        &config.autonomous.log_level,
    )?;

    Ok(())
}

/// Configure agent retry and verification behavior
pub fn configure_agent(config: &mut Config) -> Result<()> {
    display_section(
        "Agent Behavior",
        "Fine-tune how the AI agent approaches tasks",
    );

    display_hint("Retries before triggering research mode");
    config.agent.max_retry_attempts = Input::new()
        .with_prompt("Max retry attempts")
        .default(config.agent.max_retry_attempts)
        .interact()?;

    display_hint("Research attempts before giving up");
    config.agent.max_research_attempts = Input::new()
        .with_prompt("Max research attempts")
        .default(config.agent.max_research_attempts)
        .interact()?;

    display_hint("Previously-passing features to re-verify each session");
    config.agent.verification_sample_size = Input::new()
        .with_prompt("Verification sample size")
        .default(config.agent.verification_sample_size)
        .interact()?;

    display_hint("Focus on one feature at a time (recommended)");
    config.agent.single_feature_focus = Confirm::new()
        .with_prompt("Single feature focus?")
        .default(config.agent.single_feature_focus)
        .interact()?;

    Ok(())
}

/// Configure stuck recovery via alternative approaches
pub fn configure_stuck_recovery(config: &mut Config) -> Result<()> {
    display_section(
        "Stuck Recovery",
        "Generate alternative approaches when the agent gets stuck",
    );

    display_hint("Enable alternative approach generation");
    config.alternative_approaches.enabled = Confirm::new()
        .with_prompt("Enable alternative approaches?")
        .default(config.alternative_approaches.enabled)
        .interact()?;

    if config.alternative_approaches.enabled {
        display_hint("Retries before triggering alternative generation");
        config.alternative_approaches.retry_threshold = Input::new()
            .with_prompt("Retry threshold")
            .default(config.alternative_approaches.retry_threshold)
            .interact()?;

        display_hint("Number of alternative approaches to generate");
        config.alternative_approaches.num_approaches = Input::new()
            .with_prompt("Number of approaches")
            .default(config.alternative_approaches.num_approaches)
            .interact()?;

        display_hint("Cache approaches to avoid regenerating on restart");
        config.alternative_approaches.cache_results = Confirm::new()
            .with_prompt("Cache results?")
            .default(config.alternative_approaches.cache_results)
            .interact()?;
    }

    Ok(())
}

/// Configure MCP (Model Context Protocol) tools
pub fn configure_mcp(config: &mut Config) -> Result<()> {
    display_section("MCP Tools", "Configure MCP tools for enhanced capabilities");

    display_hint("Use osgrep (semantic search) instead of grep");
    config.mcp.prefer_osgrep = Confirm::new()
        .with_prompt("Prefer osgrep over grep?")
        .default(config.mcp.prefer_osgrep)
        .interact()?;

    display_hint("Use sequential thinking for complex decisions");
    config.mcp.use_sequential_thinking = Confirm::new()
        .with_prompt("Use sequential thinking?")
        .default(config.mcp.use_sequential_thinking)
        .interact()?;

    display_hint("Required MCP tools (e.g., 'chrome-devtools' for web projects)");
    let current_tools = config.mcp.required_tools.join(", ");
    let tools_str: String = Input::new()
        .with_prompt("Required tools (comma-separated)")
        .default(current_tools)
        .interact_text()?;
    config.mcp.required_tools = parse_comma_list(&tools_str);

    display_hint("Priority order for MCP tools (e.g., 'search, file-edit')");
    let current_priority = config.mcp.priority_order.join(", ");
    let priority_str: String = Input::new()
        .with_prompt("Priority order (comma-separated)")
        .default(current_priority)
        .interact_text()?;
    config.mcp.priority_order = parse_comma_list(&priority_str);

    Ok(())
}

/// Configure conductor-style planning
pub fn configure_conductor(config: &mut Config) -> Result<()> {
    display_section("Conductor", "Context-driven planning configuration");

    display_hint("Directory for context files (product.md, tech_stack.md, etc.)");
    config.conductor.context_dir = Input::new()
        .with_prompt("Context directory")
        .default(config.conductor.context_dir.clone())
        .interact_text()?;

    display_hint("Directory for track-based work units");
    config.conductor.tracks_dir = Input::new()
        .with_prompt("Tracks directory")
        .default(config.conductor.tracks_dir.clone())
        .interact_text()?;

    display_hint("Automatically set up conductor files on first run");
    config.conductor.auto_setup = Confirm::new()
        .with_prompt("Auto-setup conductor files?")
        .default(config.conductor.auto_setup)
        .interact()?;

    display_hint("Planning mode: 'auto' (AI generates) or 'manual' (user writes)");
    config.conductor.planning_mode = prompt_list_selection(
        "Planning mode",
        &["auto", "manual"],
        &config.conductor.planning_mode,
    )?;

    display_hint("Save checkpoint after completing N tasks");
    config.conductor.checkpoint_frequency = Input::new()
        .with_prompt("Checkpoint frequency")
        .default(config.conductor.checkpoint_frequency)
        .interact()?;

    Ok(())
}

/// Configure spec generation settings
pub fn configure_generation(config: &mut Config) -> Result<()> {
    display_section("Spec Generation", "Configure how app specs are generated");

    display_hint("Complexity level: 'comprehensive' or 'minimal'");
    config.generation.complexity = prompt_list_selection(
        "Complexity",
        &["comprehensive", "minimal"],
        &config.generation.complexity,
    )?;

    if config.generation.complexity == "comprehensive" {
        display_hint("Minimum features for comprehensive mode");
        config.generation.min_features = Input::new()
            .with_prompt("Min features")
            .default(config.generation.min_features)
            .interact()?;

        display_hint("Minimum implementation steps");
        config.generation.min_implementation_steps = Input::new()
            .with_prompt("Min steps")
            .default(config.generation.min_implementation_steps)
            .interact()?;
    }

    display_hint("Enable parallel sub-agent generation (faster but uses more tokens)");
    config.generation.enable_subagents = Confirm::new()
        .with_prompt("Enable sub-agents?")
        .default(config.generation.enable_subagents)
        .interact()?;

    display_hint("Include security considerations section");
    config.generation.include_security_section = Confirm::new()
        .with_prompt("Include security section?")
        .default(config.generation.include_security_section)
        .interact()?;

    display_hint("Include testing strategy section");
    config.generation.include_testing_strategy = Confirm::new()
        .with_prompt("Include testing strategy?")
        .default(config.generation.include_testing_strategy)
        .interact()?;

    display_hint("Include DevOps/deployment section");
    config.generation.include_devops_section = Confirm::new()
        .with_prompt("Include DevOps section?")
        .default(config.generation.include_devops_section)
        .interact()?;

    display_hint("Include accessibility requirements");
    config.generation.include_accessibility = Confirm::new()
        .with_prompt("Include accessibility?")
        .default(config.generation.include_accessibility)
        .interact()?;

    display_hint("Include future enhancements section");
    config.generation.include_future_enhancements = Confirm::new()
        .with_prompt("Include future enhancements?")
        .default(config.generation.include_future_enhancements)
        .interact()?;

    Ok(())
}

/// Configure security allowlist settings
pub fn configure_security(config: &mut Config) -> Result<()> {
    display_section("Security", "Configure security constraints");

    display_hint("Strictly enforce security allowlist (block dangerous commands)");
    config.security.enforce_allowlist = Confirm::new()
        .with_prompt("Enforce allowlist?")
        .default(config.security.enforce_allowlist)
        .interact()?;

    display_hint("Path to security allowlist JSON file");
    config.security.allowlist_file = Input::new()
        .with_prompt("Allowlist file")
        .default(config.security.allowlist_file.clone())
        .interact_text()?;

    Ok(())
}

/// Configure webhook notifications
pub fn configure_notifications(config: &mut Config) -> Result<()> {
    display_section(
        "Notifications",
        "Webhook notifications for feature completions",
    );

    display_hint("Send webhook notifications when features complete");
    config.notifications.webhook_enabled = Confirm::new()
        .with_prompt("Enable webhook notifications?")
        .default(config.notifications.webhook_enabled)
        .interact()?;

    if config.notifications.webhook_enabled {
        display_hint("Webhook URL to POST notifications to (Discord/Slack)");
        let current_url = config.notifications.webhook_url.clone().unwrap_or_default();
        let url: String = Input::new()
            .with_prompt("Webhook URL")
            .default(current_url)
            .interact_text()?;
        config.notifications.webhook_url = if url.is_empty() { None } else { Some(url) };
    }

    Ok(())
}

/// Configure UI display preferences
pub fn configure_ui(config: &mut Config) -> Result<()> {
    display_section("UI Settings", "Display and output preferences");

    display_hint("Show colored terminal output (disable for plain logs)");
    config.ui.colored_output = Confirm::new()
        .with_prompt("Colored output?")
        .default(config.ui.colored_output)
        .interact()?;

    display_hint("Show verbose debug output");
    config.ui.verbose = Confirm::new()
        .with_prompt("Verbose mode?")
        .default(config.ui.verbose)
        .interact()?;

    display_hint("Show progress indicators during long operations");
    config.ui.show_progress = Confirm::new()
        .with_prompt("Show progress?")
        .default(config.ui.show_progress)
        .interact()?;

    display_hint("Lines to show in spec preview");
    config.ui.spec_preview_lines = Input::new()
        .with_prompt("Spec preview lines")
        .default(config.ui.spec_preview_lines)
        .interact()?;

    Ok(())
}

/// Configure communication (Agent-User Channel)
pub fn configure_communication(config: &mut Config) -> Result<()> {
    display_section(
        "Communication",
        "Configure the agent-user communication channel",
    );

    display_hint("Enable the communication channel");
    config.communication.enabled = Confirm::new()
        .with_prompt("Enable communication?")
        .default(config.communication.enabled)
        .interact()?;

    if config.communication.enabled {
        display_hint("Path to the communication markdown file");
        config.communication.file_path = Input::new()
            .with_prompt("Communication file path")
            .default(config.communication.file_path.clone())
            .interact_text()?;

        display_hint("Automatically ask user when errors repeat");
        config.communication.auto_ask_on_error = Confirm::new()
            .with_prompt("Auto-ask on error?")
            .default(config.communication.auto_ask_on_error)
            .interact()?;

        display_hint("Check for user responses every N sessions");
        config.communication.check_interval_sessions = Input::new()
            .with_prompt("Check interval (sessions)")
            .default(config.communication.check_interval_sessions)
            .interact()?;

        display_hint("Maximum pending questions allowed");
        config.communication.max_pending_questions = Input::new()
            .with_prompt("Max pending questions")
            .default(config.communication.max_pending_questions)
            .interact()?;
    }

    Ok(())
}

/// Configure project features and priorities
pub fn configure_features(config: &mut Config) -> Result<()> {
    display_section("Features", "Feature categories and testing priorities");

    display_hint("Require a verification_command for every feature");
    config.features.require_verification_command = Confirm::new()
        .with_prompt("Require verification command?")
        .default(config.features.require_verification_command)
        .interact()?;

    display_hint("Feature categories (comma-separated)");
    let current_cats = config.features.categories.join(", ");
    let cats_str: String = Input::new()
        .with_prompt("Categories")
        .default(current_cats)
        .interact_text()?;
    config.features.categories = parse_comma_list(&cats_str);

    display_hint("Priority levels (comma-separated)");
    let current_prios = config.features.priorities.join(", ");
    let prios_str: String = Input::new()
        .with_prompt("Priorities")
        .default(current_prios)
        .interact_text()?;
    config.features.priorities = parse_comma_list(&prios_str);

    display_hint("Minimum steps for narrow tests");
    config.features.narrow_test_min_steps = Input::new()
        .with_prompt("Narrow test min steps")
        .default(config.features.narrow_test_min_steps)
        .interact()?;

    display_hint("Maximum steps for narrow tests");
    config.features.narrow_test_max_steps = Input::new()
        .with_prompt("Narrow test max steps")
        .default(config.features.narrow_test_max_steps)
        .interact()?;

    display_hint("Minimum steps for comprehensive tests");
    config.features.comprehensive_test_min_steps = Input::new()
        .with_prompt("Comprehensive test min steps")
        .default(config.features.comprehensive_test_min_steps)
        .interact()?;

    Ok(())
}

/// Configure project scaffolding settings
pub fn configure_scaffolding(config: &mut Config) -> Result<()> {
    display_section("Scaffolding", "Project initialization settings");

    display_hint("Initialize a git repository");
    config.scaffolding.git_init = Confirm::new()
        .with_prompt("Git init?")
        .default(config.scaffolding.git_init)
        .interact()?;

    display_hint("Create .opencode directory");
    config.scaffolding.create_opencode_dir = Confirm::new()
        .with_prompt("Create .opencode dir?")
        .default(config.scaffolding.create_opencode_dir)
        .interact()?;

    display_hint("Create scripts directory");
    config.scaffolding.create_scripts_dir = Confirm::new()
        .with_prompt("Create scripts dir?")
        .default(config.scaffolding.create_scripts_dir)
        .interact()?;

    display_hint("Default output directory (leave empty for current)");
    config.scaffolding.output_dir = Input::new()
        .with_prompt("Output directory")
        .default(config.scaffolding.output_dir.clone())
        .interact_text()?;

    Ok(())
}
