//! Scaffolding logic - generates files from templates

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::db;
use crate::utils::write_file;

/// Embedded default app spec template
const DEFAULT_APP_SPEC: &str = include_str!("../docs/examples/default_app_spec.md");

/// Embedded command templates
const AUTO_INIT_TEMPLATE: &str = include_str!("../templates/commands/auto-init.md");
const AUTO_CONTINUE_TEMPLATE: &str = include_str!("../templates/commands/auto-continue.md");
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
const USER_CONFIG_TEMPLATE: &str = include_str!("../templates/forger-user.toml");

/// Embedded subagent templates for parallel spec generation
const SPEC_PRODUCT_AGENT: &str = include_str!("../templates/scaffold/agents/spec-product.md");
const SPEC_ARCHITECTURE_AGENT: &str =
    include_str!("../templates/scaffold/agents/spec-architecture.md");
const SPEC_QUALITY_AGENT: &str = include_str!("../templates/scaffold/agents/spec-quality.md");

/// Embedded coder subagent for dual-model architecture
const CODER_AGENT: &str = include_str!("../templates/scaffold/agents/coder.md");

/// Embedded AGENTS.md template
const AGENTS_MD_TEMPLATE: &str = include_str!("../templates/AGENTS.md");

/// Resolve {{INCLUDE path}} directives in templates.
///
/// This implements the "Progressive Discovery" pattern, where high-level
/// command templates (like auto-init.md) include modular core logic
/// (like identity.md or security.md) only when needed by the agent.
pub fn resolve_includes(template: &str) -> String {
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

    // Write app_spec.md inside .forger/
    let spec_path = forger_dir.join("app_spec.md");
    write_file(&spec_path, spec_content)?;
    println!("   📄 Created .forger/app_spec.md");

    // Load config
    let _config = crate::config::Config::load(None).unwrap_or_default();

    // Run validation to get stats for prompt injection
    let stats = crate::validation::validate_spec(spec_content).unwrap_or_default();

    // Write command files (these stay in .opencode/ for OpenCode compatibility)
    // Templates are processed to resolve {{INCLUDE}} directives
    let auto_init_path = command_dir.join("auto-init.md");
    let auto_init_content = resolve_includes(AUTO_INIT_TEMPLATE)
        .replace(
            "{{SPEC_FEATURE_COUNT}}",
            &stats.stats.feature_count.to_string(),
        )
        .replace(
            "{{SPEC_ENDPOINT_COUNT}}",
            &stats.stats.endpoint_count.to_string(),
        );
    write_file(&auto_init_path, &auto_init_content)?;
    println!("   📄 Created .opencode/command/auto-init.md");

    let auto_continue_path = command_dir.join("auto-continue.md");
    let auto_continue_content = resolve_includes(AUTO_CONTINUE_TEMPLATE);
    write_file(&auto_continue_path, &auto_continue_content)?;
    println!("   📄 Created .opencode/command/auto-continue.md");

    let auto_enhance_path = command_dir.join("auto-enhance.md");
    write_file(&auto_enhance_path, AUTO_ENHANCE_TEMPLATE)?;
    println!("   📄 Created .opencode/command/auto-enhance.md");

    // Write security allowlist inside .forger/
    let allowlist_path = forger_dir.join("security-allowlist.json");
    write_file(&allowlist_path, SECURITY_ALLOWLIST)?;
    println!("   📄 Created .forger/security-allowlist.json");

    // Initialize SQLite database inside .forger/
    let db_path = forger_dir.join("progress.db");
    db::Database::open(&db_path)
        .with_context(|| format!("Failed to initialize database: {}", db_path.display()))?;
    println!("   🗃️  Created .forger/progress.db");

    // Write user configuration file inside .forger/ (if not already configured)
    let config_path = forger_dir.join("config.toml");
    if !config_path.exists() {
        write_file(&config_path, USER_CONFIG_TEMPLATE)?;
        println!("   ⚙️  Created .forger/config.toml");
    } else {
        println!("   ⚙️  Using existing .forger/config.toml");
    }

    // Write subagent definitions for parallel spec generation
    let spec_product_path = agent_dir.join("spec-product.md");
    write_file(&spec_product_path, SPEC_PRODUCT_AGENT)?;
    println!("   🤖 Created .opencode/agent/spec-product.md");

    let spec_arch_path = agent_dir.join("spec-architecture.md");
    write_file(&spec_arch_path, SPEC_ARCHITECTURE_AGENT)?;
    println!("   🤖 Created .opencode/agent/spec-architecture.md");

    let spec_quality_path = agent_dir.join("spec-quality.md");
    write_file(&spec_quality_path, SPEC_QUALITY_AGENT)?;
    println!("   🤖 Created .opencode/agent/spec-quality.md");

    // Write coder subagent for dual-model architecture
    let coder_path = agent_dir.join("coder.md");
    write_file(&coder_path, CODER_AGENT)?;
    println!("   🤖 Created .opencode/agent/coder.md");

    // Write .gitignore at project root if it doesn't exist
    let gitignore_path = output_dir.join(".gitignore");
    if !gitignore_path.exists() {
        let gitignore_content = generate_gitignore();
        write_file(&gitignore_path, &gitignore_content)?;
        println!("   📝 Created .gitignore");
    }

    // Write opencode.json at project root (required by OpenCode)
    let opencode_json_path = output_dir.join("opencode.json");
    let opencode_json_content = generate_opencode_json();
    write_file(&opencode_json_path, &opencode_json_content)?;
    println!("   ⚙️  Created opencode.json");

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
    let forger_dir = output_dir.join(".forger");
    let opencode_dir = output_dir.join(".opencode");
    let command_dir = opencode_dir.join("command");

    println!("\n📋 Preview: Files that would be created");
    println!("{}", "─".repeat(50));

    // Directories
    println!("\nDirectories:");
    println!("   📁 {}", forger_dir.display());
    println!("   📁 {}", opencode_dir.display());
    println!("   📁 {}", command_dir.display());

    // Files
    println!("\nFiles:");
    println!("   📄 {}", forger_dir.join("app_spec.md").display());
    println!("   📄 {}", command_dir.join("auto-init.md").display());
    println!("   📄 {}", command_dir.join("auto-continue.md").display());
    println!("   📄 {}", command_dir.join("auto-enhance.md").display());
    println!(
        "   📄 {}",
        forger_dir.join("security-allowlist.json").display()
    );
    println!("   🗃️  {}", forger_dir.join("progress.db").display());
    println!("   ⚙️  {}", forger_dir.join("config.toml").display());
    println!("   ⚙️  {}", output_dir.join("opencode.json").display());
    println!("   📝 {}", output_dir.join("AGENTS.md").display());

    println!("\n{}", "─".repeat(50));
    println!("Total: 3 directories, 8 files");
    println!("Run without --dry-run to create these files.");
}

/// Generate the opencode.json content with default MCP settings
/// Note: This generates valid JSON (no comments) since .json files don't support JSONC
fn generate_opencode_json() -> String {
    r#"{
  "$schema": "https://opencode.ai/config.json",
  "instructions": [
    ".forger/config.toml",
    ".forger/app_spec.md"
  ],
  "provider": {
    "google": {
      "options": {
        "thinkingLevel": "high"
      }
    },
    "anthropic": {
      "options": {
        "thinking": {
          "type": "enabled",
          "budget_tokens": 16000
        }
      }
    },
    "deepseek": {
      "options": {
        "thinking": {
          "type": "enabled"
        }
      }
    }
  },
  "mcp": {},
  "permission": {
  "*": "allow",
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
.approach-cache/
.vs-cache/

# OpenCode Forger
progress.db*
opencode-debug.log
.forger/logs/
.conductor/
"#
    .to_string()
}
