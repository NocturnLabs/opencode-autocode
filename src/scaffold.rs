//! Scaffolding logic - generates files from templates

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::communication::CommunicationChannel;
use crate::db;

/// Embedded default app spec template
const DEFAULT_APP_SPEC: &str = include_str!("../docs/examples/default_app_spec.md");

/// Embedded command templates (v2 modular versions)
const AUTO_INIT_TEMPLATE: &str = include_str!("../templates/commands/auto-init-v2.md");
const AUTO_CONTINUE_TEMPLATE: &str = include_str!("../templates/commands/auto-continue-v2.md");
const AUTO_ENHANCE_TEMPLATE: &str = include_str!("../templates/commands/auto-enhance.md");

/// Core modules for include directive resolution
const CORE_IDENTITY: &str = include_str!("../templates/core/identity.md");
const CORE_SECURITY: &str = include_str!("../templates/core/security.md");
const CORE_SIGNALING: &str = include_str!("../templates/core/signaling.md");
const CORE_DATABASE: &str = include_str!("../templates/core/database.md");
const CORE_COMMUNICATION: &str = include_str!("../templates/core/communication.md");
const CORE_MCP_GUIDE: &str = include_str!("../templates/core/mcp_guide.md");

/// Embedded security allowlist
const SECURITY_ALLOWLIST: &str = include_str!("../templates/scripts/security-allowlist.json");

/// Embedded user configuration template
const USER_CONFIG_TEMPLATE: &str = include_str!("../templates/autocode-user.toml");

/// Embedded subagent templates for parallel spec generation
const SPEC_PRODUCT_AGENT: &str = include_str!("../templates/scaffold/agents/spec-product.md");
const SPEC_ARCHITECTURE_AGENT: &str =
    include_str!("../templates/scaffold/agents/spec-architecture.md");
const SPEC_QUALITY_AGENT: &str = include_str!("../templates/scaffold/agents/spec-quality.md");

/// Resolve {{INCLUDE path}} directives in templates
/// Replaces include directives with the actual content of the referenced modules
/// Resolve {{INCLUDE path}} directives in templates.
///
/// This implements the "Progressive Discovery" pattern, where high-level
/// command templates (like auto-init.md) include modular core logic
/// (like identity.md or security.md) only when needed by the agent.
fn resolve_includes(template: &str) -> String {
    let mut result = template.to_string();

    // Map of include paths to their embedded content
    let includes: &[(&str, &str)] = &[
        ("core/identity.md", CORE_IDENTITY),
        ("core/security.md", CORE_SECURITY),
        ("core/signaling.md", CORE_SIGNALING),
        ("core/database.md", CORE_DATABASE),
        ("core/communication.md", CORE_COMMUNICATION),
        ("core/mcp_guide.md", CORE_MCP_GUIDE),
    ];

    for (path, content) in includes {
        let directive = format!("{{{{INCLUDE {}}}}}", path);
        result = result.replace(&directive, content);
    }

    result
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
    let spec_content = fs::read_to_string(spec_path)
        .with_context(|| format!("Failed to read spec file: {}", spec_path.display()))?;
    scaffold_with_spec_text(output_dir, &spec_content)
}

/// Scaffold with raw spec text (used by AI-generated spec flow)
pub fn scaffold_with_spec_text(output_dir: &Path, spec_content: &str) -> Result<()> {
    debug_assert!(!spec_content.is_empty(), "Spec content cannot be empty");
    debug_assert!(
        !output_dir.as_os_str().is_empty(),
        "Output dir cannot be empty"
    );

    // Create .autocode parent directory for all autocode-managed files
    let autocode_dir = output_dir.join(".autocode");
    let opencode_dir = output_dir.join(".opencode");
    let command_dir = opencode_dir.join("command"); // OpenCode expects this at .opencode/command

    fs::create_dir_all(&autocode_dir).with_context(|| {
        format!(
            "Failed to create .autocode directory: {}",
            autocode_dir.display()
        )
    })?;
    fs::create_dir_all(&command_dir).with_context(|| {
        format!(
            "Failed to create command directory: {}",
            command_dir.display()
        )
    })?;

    // Create .opencode/agent directory for subagent definitions
    let agent_dir = opencode_dir.join("agent");
    fs::create_dir_all(&agent_dir)
        .with_context(|| format!("Failed to create agent directory: {}", agent_dir.display()))?;

    // Write app_spec.md inside .autocode/
    let spec_path = autocode_dir.join("app_spec.md");
    fs::write(&spec_path, spec_content)
        .with_context(|| format!("Failed to write app_spec.md: {}", spec_path.display()))?;
    println!("   📄 Created .autocode/app_spec.md");

    // Write command files (these stay in .opencode/ for OpenCode compatibility)
    // Templates are processed to resolve {{INCLUDE}} directives
    let auto_init_path = command_dir.join("auto-init.md");
    let auto_init_content = resolve_includes(AUTO_INIT_TEMPLATE);
    fs::write(&auto_init_path, auto_init_content)
        .with_context(|| format!("Failed to write auto-init.md: {}", auto_init_path.display()))?;
    println!("   📄 Created .opencode/command/auto-init.md");

    let auto_continue_path = command_dir.join("auto-continue.md");
    let auto_continue_content = resolve_includes(AUTO_CONTINUE_TEMPLATE);
    fs::write(&auto_continue_path, auto_continue_content).with_context(|| {
        format!(
            "Failed to write auto-continue.md: {}",
            auto_continue_path.display()
        )
    })?;
    println!("   📄 Created .opencode/command/auto-continue.md");

    let auto_enhance_path = command_dir.join("auto-enhance.md");
    fs::write(&auto_enhance_path, AUTO_ENHANCE_TEMPLATE).with_context(|| {
        format!(
            "Failed to write auto-enhance.md: {}",
            auto_enhance_path.display()
        )
    })?;
    println!("   📄 Created .opencode/command/auto-enhance.md");

    // Write security allowlist inside .autocode/
    let allowlist_path = autocode_dir.join("security-allowlist.json");
    fs::write(&allowlist_path, SECURITY_ALLOWLIST).with_context(|| {
        format!(
            "Failed to write security-allowlist.json: {}",
            allowlist_path.display()
        )
    })?;
    println!("   📄 Created .autocode/security-allowlist.json");

    // Initialize SQLite database inside .autocode/
    let db_path = autocode_dir.join("progress.db");
    db::Database::open(&db_path)
        .with_context(|| format!("Failed to initialize database: {}", db_path.display()))?;
    println!("   🗃️  Created .autocode/progress.db");

    // Write user configuration file inside .autocode/
    let config_path = autocode_dir.join("config.toml");
    fs::write(&config_path, USER_CONFIG_TEMPLATE)
        .with_context(|| format!("Failed to write config.toml: {}", config_path.display()))?;
    println!("   ⚙️  Created .autocode/config.toml");

    // Write subagent definitions for parallel spec generation
    let spec_product_path = agent_dir.join("spec-product.md");
    fs::write(&spec_product_path, SPEC_PRODUCT_AGENT).with_context(|| {
        format!(
            "Failed to write spec-product.md: {}",
            spec_product_path.display()
        )
    })?;
    println!("   🤖 Created .opencode/agent/spec-product.md");

    let spec_arch_path = agent_dir.join("spec-architecture.md");
    fs::write(&spec_arch_path, SPEC_ARCHITECTURE_AGENT).with_context(|| {
        format!(
            "Failed to write spec-architecture.md: {}",
            spec_arch_path.display()
        )
    })?;
    println!("   🤖 Created .opencode/agent/spec-architecture.md");

    let spec_quality_path = agent_dir.join("spec-quality.md");
    fs::write(&spec_quality_path, SPEC_QUALITY_AGENT).with_context(|| {
        format!(
            "Failed to write spec-quality.md: {}",
            spec_quality_path.display()
        )
    })?;
    println!("   🤖 Created .opencode/agent/spec-quality.md");

    // Initialize communication channel
    let comm_channel = CommunicationChannel::new(&autocode_dir.join("COMMUNICATION.md"));
    comm_channel.init()?;
    println!("   💬 Created .autocode/COMMUNICATION.md");

    // Write .gitignore at project root if it doesn't exist
    let gitignore_path = output_dir.join(".gitignore");
    if !gitignore_path.exists() {
        let gitignore_content = generate_gitignore();
        fs::write(&gitignore_path, gitignore_content)
            .with_context(|| format!("Failed to write .gitignore: {}", gitignore_path.display()))?;
        println!("   📝 Created .gitignore");
    }

    // Write opencode.json at project root (required by OpenCode)
    let opencode_json_path = output_dir.join("opencode.json");
    let opencode_json_content = generate_opencode_json();
    fs::write(&opencode_json_path, opencode_json_content).with_context(|| {
        format!(
            "Failed to write opencode.json: {}",
            opencode_json_path.display()
        )
    })?;
    println!("   ⚙️  Created opencode.json");

    Ok(())
}

/// Scaffold with an AppSpec struct (used by TUI)
pub fn scaffold_from_spec(output_dir: &Path, spec: &crate::spec::AppSpec) -> Result<()> {
    let spec_text = spec.to_spec_text();
    scaffold_with_spec_text(output_dir, &spec_text)
}

/// Preview what files would be created without actually creating them
pub fn preview_scaffold(output_dir: &Path) {
    use console::style;

    let autocode_dir = output_dir.join(".autocode");
    let opencode_dir = output_dir.join(".opencode");
    let command_dir = opencode_dir.join("command");

    println!(
        "\n{}",
        style("📋 Preview: Files that would be created")
            .cyan()
            .bold()
    );
    println!("{}", style("─".repeat(50)).dim());

    // Directories
    println!("\n{}", style("Directories:").yellow());
    println!("   📁 {}", style(autocode_dir.display()).dim());
    println!("   📁 {}", style(opencode_dir.display()).dim());
    println!("   📁 {}", style(command_dir.display()).dim());

    // Files
    println!("\n{}", style("Files:").yellow());
    println!(
        "   📄 {}",
        style(autocode_dir.join("app_spec.md").display()).green()
    );
    println!(
        "   📄 {}",
        style(command_dir.join("auto-init.md").display()).green()
    );
    println!(
        "   📄 {}",
        style(command_dir.join("auto-continue.md").display()).green()
    );
    println!(
        "   📄 {}",
        style(command_dir.join("auto-enhance.md").display()).green()
    );
    println!(
        "   📄 {}",
        style(autocode_dir.join("security-allowlist.json").display()).green()
    );
    println!(
        "   🗃️  {}",
        style(autocode_dir.join("progress.db").display()).green()
    );
    println!(
        "   ⚙️  {}",
        style(autocode_dir.join("config.toml").display()).green()
    );
    println!(
        "   ⚙️  {}",
        style(output_dir.join("opencode.json").display()).green()
    );

    println!("\n{}", style("─".repeat(50)).dim());
    println!("{}", style("Total: 3 directories, 8 files").cyan());
    println!(
        "{}",
        style("Run without --dry-run to create these files.").dim()
    );
}

/// Generate the opencode.json content with default MCP settings
/// Note: This generates valid JSON (no comments) since .json files don't support JSONC
fn generate_opencode_json() -> String {
    r#"{
  "$schema": "https://opencode.ai/config.json",
  "instructions": [
    ".autocode/config.toml",
    ".autocode/app_spec.md"
  ],
  "mcp": {},
  "permission": {
    "bash": "allow",
    "edit": "allow"
  }
}
"#
    .to_string()
}

/// Generate .gitignore content for scaffolded projects
/// Only includes essential tool-related entries - AI generates the rest based on project type
fn generate_gitignore() -> String {
    r#"# MCP & Tool caches (do not commit)
.osgrep/
"#
    .to_string()
}
