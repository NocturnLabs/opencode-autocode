//! Scaffolding logic - generates files from templates

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Embedded default app spec template
const DEFAULT_APP_SPEC: &str = include_str!("../default_app_spec.md");

/// Embedded command templates
const AUTO_INIT_TEMPLATE: &str = include_str!("../templates/commands/auto-init.md");
const AUTO_CONTINUE_TEMPLATE: &str = include_str!("../templates/commands/auto-continue.md");
const AUTO_ENHANCE_TEMPLATE: &str = include_str!("../templates/commands/auto-enhance.md");

/// Embedded security allowlist
const SECURITY_ALLOWLIST: &str = include_str!("../templates/scripts/security-allowlist.json");

/// Embedded user configuration template
const USER_CONFIG_TEMPLATE: &str = include_str!("../templates/autocode-user.toml");

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
    // Create directory structure
    let opencode_dir = output_dir.join(".opencode");
    let command_dir = opencode_dir.join("command"); // OpenCode expects singular "command"
    let scripts_dir = output_dir.join("scripts");

    fs::create_dir_all(&command_dir).with_context(|| {
        format!(
            "Failed to create command directory: {}",
            command_dir.display()
        )
    })?;
    fs::create_dir_all(&scripts_dir).with_context(|| {
        format!(
            "Failed to create scripts directory: {}",
            scripts_dir.display()
        )
    })?;

    // Write app_spec.md
    let spec_path = output_dir.join("app_spec.md");
    fs::write(&spec_path, spec_content)
        .with_context(|| format!("Failed to write app_spec.md: {}", spec_path.display()))?;
    println!("   📄 Created app_spec.md");

    // Write command files
    let auto_init_path = command_dir.join("auto-init.md");
    fs::write(&auto_init_path, AUTO_INIT_TEMPLATE)
        .with_context(|| format!("Failed to write auto-init.md: {}", auto_init_path.display()))?;
    println!("   📄 Created .opencode/command/auto-init.md");

    let auto_continue_path = command_dir.join("auto-continue.md");
    fs::write(&auto_continue_path, AUTO_CONTINUE_TEMPLATE).with_context(|| {
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

    // Write security allowlist
    let allowlist_path = scripts_dir.join("security-allowlist.json");
    fs::write(&allowlist_path, SECURITY_ALLOWLIST).with_context(|| {
        format!(
            "Failed to write security-allowlist.json: {}",
            allowlist_path.display()
        )
    })?;
    println!("   📄 Created scripts/security-allowlist.json");

    // Create empty progress file
    let progress_path = output_dir.join("opencode-progress.txt");
    fs::write(
        &progress_path,
        "# OpenCode Autonomous Progress\n\nNo sessions completed yet.\n",
    )
    .with_context(|| {
        format!(
            "Failed to write opencode-progress.txt: {}",
            progress_path.display()
        )
    })?;
    println!("   📄 Created opencode-progress.txt");

    // Write user configuration file
    let config_path = output_dir.join("autocode.toml");
    fs::write(&config_path, USER_CONFIG_TEMPLATE)
        .with_context(|| format!("Failed to write autocode.toml: {}", config_path.display()))?;
    println!("   ⚙️  Created autocode.toml");

    // Write opencode.json for OpenCode native configuration
    let opencode_json_path = output_dir.join("opencode.json");
    let opencode_json_content = generate_opencode_json();
    fs::write(&opencode_json_path, opencode_json_content)
        .with_context(|| format!("Failed to write opencode.json: {}", opencode_json_path.display()))?;
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

    let opencode_dir = output_dir.join(".opencode");
    let command_dir = opencode_dir.join("command");
    let scripts_dir = output_dir.join("scripts");

    println!(
        "\n{}",
        style("📋 Preview: Files that would be created")
            .cyan()
            .bold()
    );
    println!("{}", style("─".repeat(50)).dim());

    // Directories
    println!("\n{}", style("Directories:").yellow());
    println!("   📁 {}", style(opencode_dir.display()).dim());
    println!("   📁 {}", style(command_dir.display()).dim());
    println!("   📁 {}", style(scripts_dir.display()).dim());

    // Files
    println!("\n{}", style("Files:").yellow());
    println!(
        "   📄 {}",
        style(output_dir.join("app_spec.md").display()).green()
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
        style(scripts_dir.join("security-allowlist.json").display()).green()
    );
    println!(
        "   📄 {}",
        style(output_dir.join("opencode-progress.txt").display()).green()
    );
    println!(
        "   ⚙️  {}",
        style(output_dir.join("autocode.toml").display()).green()
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
fn generate_opencode_json() -> String {
    r#"{
  "$schema": "https://opencode.ai/config.json",
  
  // OpenCode configuration generated by opencode-autocode
  // Edit this file to configure OpenCode behavior for this project
  
  // Instruction files for the AI to read
  "instructions": [
    "autocode.toml",
    "app_spec.md"
  ],
  
  // MCP server configurations
  // Enable/disable MCP tools based on your installed servers
  // See: https://opencode.ai/docs/mcp-servers
  "mcp": {
    // Semantic code search - faster than grep for large codebases
    // Install: https://github.com/Ryandonofrio3/osgrep
    "osgrep": {
      "type": "local",
      "command": ["osgrep", "mcp"],
      "enabled": false
    },
    
    // Sequential thinking for complex reasoning
    // Install: npx -y @anthropic/thinking-mcp
    "sequential-thinking": {
      "type": "local", 
      "command": ["npx", "-y", "@anthropic/thinking-mcp"],
      "enabled": false
    },
    
    // Chrome DevTools for web projects (browser testing)
    // Install: npx -y @anthropic/chrome-devtools-mcp
    "chrome-devtools": {
      "type": "local",
      "command": ["npx", "-y", "@anthropic/chrome-devtools-mcp"],
      "enabled": false
    }
  },
  
  // Permission settings for tool execution
  // Options: "allow" (auto-approve), "ask" (require approval)
  "permission": {
    "bash": "allow",
    "edit": "allow"
  }
}
"#.to_string()
}
