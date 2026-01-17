//! Scaffolding logic - generates files from templates
//! Refactored to use assets module.

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::db;
use crate::utils::write_file;

mod assets;
use assets::*;

/// Resolve {{INCLUDE path}} directives in templates.
///
/// This implements the "Progressive Discovery" pattern, where high-level
/// command templates (like auto-init.md) include modular core logic
/// (like identity.xml or security.xml) only when needed by the agent.
pub fn resolve_includes(template: &str) -> Result<String> {
    let mut result = template.to_string();

    let includes = [
        ("core/identity.xml", core_identity()?),
        ("core/security.xml", core_security()?),
        ("core/signaling.xml", core_signaling()?),
        ("core/database.xml", core_database()?),
        ("core/mcp_guide.xml", core_mcp_guide()?),
    ];

    for (path, content) in includes {
        let directive = format!("{{{{INCLUDE {}}}}}", path);
        result = result.replace(&directive, &content);
    }

    Ok(result)
}

/// Scaffold with the default embedded app spec
pub fn scaffold_default(output_dir: &Path) -> Result<()> {
    scaffold_with_spec_text(output_dir, DEFAULT_APP_SPEC)
}

/// Scaffold with a custom app spec file
pub fn scaffold_custom(output_dir: &Path, spec_path: &Path) -> Result<()> {
    debug_assert!(
        spec_path.exists(),
        "Spec path should exist before scaffolding"
    );
    let spec_content = crate::utils::read_file(spec_path)?;
    scaffold_with_spec_text(output_dir, &spec_content)
}

/// Scaffold with raw spec text (used by AI-generated spec flow)
pub fn scaffold_with_spec_text(output_dir: &Path, spec_content: &str) -> Result<()> {
    debug_assert!(!spec_content.is_empty(), "Spec content cannot be empty");
    debug_assert!(
        !output_dir.as_os_str().is_empty(),
        "Output dir cannot be empty"
    );

    let config = crate::config::Config::load(Some(output_dir)).unwrap_or_default();

    // Create .forger parent directory for all forger-managed files
    let forger_dir = output_dir.join(".forger");
    let opencode_dir = output_dir.join(".opencode");
    let command_dir = opencode_dir.join("command"); // OpenCode expects this at .opencode/command

    fs::create_dir_all(&forger_dir).with_context(|| {
        format!(
            "Failed to create .forger directory: {}",
            forger_dir.display()
        )
    })?;

    // Create .opencode directories only when enabled
    let agent_dir = opencode_dir.join("agent");
    if config.scaffolding.create_opencode_dir {
        fs::create_dir_all(&command_dir).with_context(|| {
            format!(
                "Failed to create command directory: {}",
                command_dir.display()
            )
        })?;
        fs::create_dir_all(&agent_dir).with_context(|| {
            format!("Failed to create agent directory: {}", agent_dir.display())
        })?;
    }

    let spec_path = resolve_scaffold_path(output_dir, &config.paths.app_spec_file);
    ensure_parent_dir(&spec_path)?;
    write_file(&spec_path, spec_content)?;
    println!("   📄 Created {}", spec_path.display());

    // Run validation to get stats for prompt injection
    let stats =
        crate::validation::validate_spec_with_config(spec_content, &config).unwrap_or_default();

    if config.scaffolding.create_opencode_dir {
        // Write command files (these stay in .opencode/ for OpenCode compatibility)
        // Templates are processed to resolve {{INCLUDE}} directives
        let auto_init_path = command_dir.join("auto-init.md");
        let auto_init_template = auto_init_template()?;
        let auto_init_content = resolve_includes(&auto_init_template)?
            .replace(
                "{{SPEC_FEATURE_COUNT}}",
                &stats.stats.feature_count.to_string(),
            )
            .replace(
                "{{SPEC_ENDPOINT_COUNT}}",
                &stats.stats.endpoint_count.to_string(),
            )
            .replace("{{APP_SPEC_PATH}}", &config.paths.app_spec_file)
            .replace(
                "{{FEATURE_CATEGORY_LIST}}",
                &config.features.categories.join(", "),
            )
            .replace(
                "{{PRIORITY_LEVEL_LIST}}",
                &config.features.priorities.join(", "),
            )
            .replace(
                "{{REQUIRE_VERIFICATION_COMMAND}}",
                if config.features.require_verification_command {
                    "Required"
                } else {
                    "Optional"
                },
            )
            .replace(
                "{{NARROW_TEST_MIN_STEPS}}",
                &config.features.narrow_test_min_steps.to_string(),
            )
            .replace(
                "{{NARROW_TEST_MAX_STEPS}}",
                &config.features.narrow_test_max_steps.to_string(),
            )
            .replace(
                "{{COMPREHENSIVE_TEST_MIN_STEPS}}",
                &config.features.comprehensive_test_min_steps.to_string(),
            )
            .replace(
                "{{MIN_FEATURES_REQUIRED}}",
                &config.generation.requirements().min_features.to_string(),
            )
            .replace(
                "{{MIN_TABLES_REQUIRED}}",
                &config
                    .generation
                    .requirements()
                    .min_database_tables
                    .to_string(),
            )
            .replace(
                "{{MIN_ENDPOINTS_REQUIRED}}",
                &config
                    .generation
                    .requirements()
                    .min_api_endpoints
                    .to_string(),
            )
            .replace(
                "{{MIN_IMPLEMENTATION_STEPS_REQUIRED}}",
                &config
                    .generation
                    .requirements()
                    .min_implementation_steps
                    .to_string(),
            );
        write_file(&auto_init_path, &auto_init_content)?;
        println!("   📄 Created .opencode/command/auto-init.md");

        let auto_continue_path = command_dir.join("auto-continue.md");
        let auto_continue_template = auto_continue_template()?;
        let auto_continue_content = resolve_includes(&auto_continue_template)?
            .replace("{{APP_SPEC_PATH}}", &config.paths.app_spec_file)
            .replace(
                "{{REGRESSION_SAMPLE_SIZE}}",
                &config.agent.verification_sample_size.to_string(),
            );
        write_file(&auto_continue_path, &auto_continue_content)?;
        println!("   📄 Created .opencode/command/auto-continue.md");

        let auto_enhance_path = command_dir.join("auto-enhance.md");
        let auto_enhance_template = auto_enhance_template()?;
        let auto_enhance_content =
            auto_enhance_template.replace("{{APP_SPEC_PATH}}", &config.paths.app_spec_file);
        write_file(&auto_enhance_path, &auto_enhance_content)?;
        println!("   📄 Created .opencode/command/auto-enhance.md");
    }

    let allowlist_path = resolve_scaffold_path(output_dir, &config.security.allowlist_file);
    ensure_parent_dir(&allowlist_path)?;
    write_file(&allowlist_path, SECURITY_ALLOWLIST)?;
    println!("   📄 Created {}", allowlist_path.display());

    let db_path = resolve_scaffold_path(output_dir, &config.paths.database_file);
    ensure_parent_dir(&db_path)?;
    db::Database::open(&db_path)
        .with_context(|| format!("Failed to initialize database: {}", db_path.display()))?;
    println!("   🗃️  Created {}", db_path.display());

    // Write user configuration file at project root (if not already configured)
    let root_config_path = output_dir.join("forger.toml");
    if !root_config_path.exists() {
        write_file(&root_config_path, USER_CONFIG_TEMPLATE)?;
        println!("   ⚙️  Created forger.toml");
    } else {
        println!("   ⚙️  Using existing forger.toml");
    }

    if config.scaffolding.create_opencode_dir {
        // Write subagent definitions for parallel spec generation
        let spec_product_path = agent_dir.join("spec-product.md");
        write_file(&spec_product_path, &spec_product_agent()?)?;
        println!("   🤖 Created .opencode/agent/spec-product.md");

        let spec_arch_path = agent_dir.join("spec-architecture.md");
        write_file(&spec_arch_path, &spec_architecture_agent()?)?;
        println!("   🤖 Created .opencode/agent/spec-architecture.md");

        let spec_quality_path = agent_dir.join("spec-quality.md");
        write_file(&spec_quality_path, &spec_quality_agent()?)?;
        println!("   🤖 Created .opencode/agent/spec-quality.md");
    }

    // Write .gitignore at project root if it doesn't exist
    let gitignore_path = output_dir.join(".gitignore");
    if !gitignore_path.exists() {
        let gitignore_content = generate_gitignore(&config);
        write_file(&gitignore_path, &gitignore_content)?;
        println!("   📝 Created .gitignore");
    }

    // Write opencode.json at project root (required by OpenCode)
    let opencode_json_path = output_dir.join("opencode.json");
    let opencode_json_content = generate_opencode_json(&config);
    write_file(&opencode_json_path, &opencode_json_content)?;
    println!("   ⚙️  Created opencode.json");

    if config.scaffolding.create_scripts_dir {
        let scripts_path = output_dir.join("scripts");
        fs::create_dir_all(&scripts_path).with_context(|| {
            format!(
                "Failed to create scripts directory: {}",
                scripts_path.display()
            )
        })?;
        println!("   📁 Created {}", scripts_path.display());
    }

    if config.scaffolding.git_init {
        let git_dir = output_dir.join(".git");
        if !git_dir.exists() {
            Command::new("git")
                .arg("init")
                .current_dir(output_dir)
                .status()
                .context("Failed to initialize git repository")?;
            println!("   🌱 Initialized git repository");
        }
    }

    // Write AGENTS.md at project root
    let agents_md_path = output_dir.join("AGENTS.md");
    if !agents_md_path.exists() {
        write_file(&agents_md_path, AGENTS_MD_TEMPLATE)?;
        println!("   📝 Created AGENTS.md");
    }

    Ok(())
}

/// Scaffold with an AppSpec struct (used by TUI)
pub fn scaffold_from_spec(output_dir: &Path, spec: &crate::spec::AppSpec) -> Result<()> {
    let spec_text = spec.to_spec_text();
    scaffold_with_spec_text(output_dir, &spec_text)
}

/// Preview what files would be created without actually creating them
pub fn preview_scaffold(output_dir: &Path) {
    let config = crate::config::Config::load(Some(output_dir)).unwrap_or_default();
    let forger_dir = output_dir.join(".forger");
    let opencode_dir = output_dir.join(".opencode");
    let command_dir = opencode_dir.join("command");

    let spec_path = resolve_scaffold_path(output_dir, &config.paths.app_spec_file);
    let allowlist_path = resolve_scaffold_path(output_dir, &config.security.allowlist_file);
    let db_path = resolve_scaffold_path(output_dir, &config.paths.database_file);

    println!("\n📋 Preview: Files that would be created");
    println!("{}", "─".repeat(50));

    // Directories
    println!("\nDirectories:");
    println!("   📁 {}", forger_dir.display());
    if config.scaffolding.create_opencode_dir {
        println!("   📁 {}", opencode_dir.display());
        println!("   📁 {}", command_dir.display());
    }
    if config.scaffolding.create_scripts_dir {
        println!("   📁 {}", output_dir.join("scripts").display());
    }

    // Files
    println!("\nFiles:");
    println!("   📄 {}", spec_path.display());
    if config.scaffolding.create_opencode_dir {
        println!("   📄 {}", command_dir.join("auto-init.md").display());
        println!("   📄 {}", command_dir.join("auto-continue.md").display());
        println!("   📄 {}", command_dir.join("auto-enhance.md").display());
    }
    println!("   📄 {}", allowlist_path.display());
    println!("   🗃️  {}", db_path.display());
    println!("   ⚙️  {}", output_dir.join("forger.toml").display());
    println!("   ⚙️  {}", output_dir.join("opencode.json").display());
    println!("   📝 {}", output_dir.join("AGENTS.md").display());

    println!("\n{}", "─".repeat(50));
    println!("Run without --dry-run to create these files.");
}

/// Generate the opencode.json content with default MCP settings.
fn generate_opencode_json(config: &crate::config::Config) -> String {
    let sequential_thinking_enabled = config.mcp.use_sequential_thinking;
    let chrome_devtools_enabled = config
        .mcp
        .required_tools
        .iter()
        .any(|t| t.eq_ignore_ascii_case("chrome-devtools"));
    let app_spec_path = if config.paths.app_spec_file.trim().is_empty() {
        ".forger/app_spec.md"
    } else {
        config.paths.app_spec_file.as_str()
    };

    format!(
        r#"{{
  "$schema": "https://opencode.ai/config.json",
  "instructions": [
    "forger.toml",
    "{app_spec_path}"
  ],
  "provider": {{
    "google": {{
      "options": {{
        "thinkingLevel": "high"
      }}
    }},
    "anthropic": {{
      "options": {{
        "thinking": {{
          "type": "enabled",
          "budget_tokens": 16000
        }}
      }}
    }},
    "deepseek": {{
      "options": {{
        "thinking": {{
          "type": "enabled"
        }}
      }}
    }}
  }},
  "mcp": {{
    "sequential-thinking": {{
      "type": "local",
      "command": ["npx", "-y", "@modelcontextprotocol/server-sequential-thinking"],
      "enabled": {sequential_thinking_enabled}
    }},
    "chrome-devtools": {{
      "type": "local",
      "command": ["npx", "-y", "chrome-devtools-mcp@latest"],
      "enabled": {chrome_devtools_enabled}
    }}
  }},
  "permission": {{
  "*": "allow",
    "bash": "allow",
    "edit": "allow"
  }}
}}
"#,
        app_spec_path = app_spec_path,
        sequential_thinking_enabled = sequential_thinking_enabled,
        chrome_devtools_enabled = chrome_devtools_enabled,
    )
}

/// Generate .gitignore content for scaffolded projects.
fn generate_gitignore(config: &crate::config::Config) -> String {
    let mut entries = vec![
        "# MCP & Tool caches (do not commit)".to_string(),
        normalize_gitignore_path(&config.alternative_approaches.cache_dir),
        normalize_gitignore_path(&config.paths.vs_cache_dir),
        "".to_string(),
        "# OpenCode Forger".to_string(),
        "progress.db*".to_string(),
        "opencode-debug*.log".to_string(),
        normalize_gitignore_path(&config.conductor.context_dir),
    ];

    let log_dir = normalize_gitignore_path(&config.paths.log_dir);
    if !log_dir.is_empty() {
        entries.push(log_dir);
    }

    entries.push(String::new());
    entries.join("\n")
}

/// @param path Raw path from configuration.
/// @returns A normalized gitignore entry for relative paths.
fn normalize_gitignore_path(path: &str) -> String {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let path = Path::new(trimmed);
    if path.is_absolute() {
        return String::new();
    }

    let mut normalized = trimmed.replace('\\', "/");
    if !normalized.ends_with('/') {
        normalized.push('/');
    }
    normalized
}

/// @param output_dir Output directory for scaffolding.
/// @param path_config Configured path string.
/// @returns An absolute path rooted at the output directory when needed.
fn resolve_scaffold_path(output_dir: &Path, path_config: &str) -> PathBuf {
    let trimmed = path_config.trim();
    if trimmed.is_empty() {
        return output_dir.to_path_buf();
    }

    let path = Path::new(trimmed);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        output_dir.join(path)
    }
}

/// @param path File path to ensure parents exist for.
/// @returns An error if the parent directory cannot be created.
fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }
    }
    Ok(())
}
