//! CLI argument parsing using clap

use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// OpenCode Autocode - Autonomous Coding for OpenCode
#[derive(Parser, Debug)]
#[command(name = "opencode-autocode")]
#[command(author = "NocturnLabs")]
#[command(version)]
#[command(about = "Scaffold and run autonomous coding projects for OpenCode", long_about = None)]
pub struct Cli {
    /// Subcommand to run
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Launch interactive TUI to build app spec (scaffolding mode)
    #[arg(long)]
    pub interactive: bool,

    /// Use the default app spec template (scaffolding mode)
    #[arg(long)]
    pub default: bool,

    /// Path to a custom app spec file (scaffolding mode)
    #[arg(long, value_name = "FILE")]
    pub spec: Option<PathBuf>,

    /// Configure settings via TUI form
    #[arg(long)]
    pub config: bool,

    /// Output directory for scaffolded files
    #[arg(short, long, value_name = "DIR")]
    pub output: Option<PathBuf>,

    /// Preview scaffolding without creating files
    #[arg(long, visible_alias = "preview")]
    pub dry_run: bool,

    /// Disable parallel subagent spec generation (use legacy single-pass)
    #[arg(long)]
    pub no_subagents: bool,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

/// Subcommands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start the autonomous coding loop
    Vibe {
        /// Maximum number of iterations (default: unlimited)
        #[arg(short, long)]
        limit: Option<usize>,

        /// Path to custom config file (default: autocode.toml)
        #[arg(long, value_name = "FILE")]
        config_file: Option<PathBuf>,

        /// Enable developer mode with comprehensive debug logging
        #[arg(long)]
        developer: bool,

        /// Use single model for all tasks (disables dual-model reasoning/coding split)
        #[arg(long)]
        single_model: bool,

        /// Run N workers in parallel using git worktrees (0 = auto-detect)
        #[arg(long, value_name = "N")]
        parallel: Option<usize>,

        /// Target a specific feature ID (used by parallel workers)
        #[arg(long)]
        feature_id: Option<i64>,
    },
    /// Start the autonomous enhancement loop (infinite refine)
    Enhance {
        /// Maximum number of iterations (default: unlimited)
        #[arg(short, long)]
        limit: Option<usize>,

        /// Path to custom config file (default: autocode.toml)
        #[arg(long, value_name = "FILE")]
        config_file: Option<PathBuf>,

        /// Enable developer mode with comprehensive debug logging
        #[arg(long)]
        developer: bool,

        /// Use single model for all tasks (disables dual-model reasoning/coding split)
        #[arg(long)]
        single_model: bool,
    },
    /// Manage project templates
    Templates {
        #[command(subcommand)]
        action: TemplateAction,
    },
    /// Database management commands
    Db {
        #[command(subcommand)]
        action: DbAction,
    },
    /// Show example documentation for various topics
    Example {
        #[command(subcommand)]
        topic: ExampleTopic,
    },
    /// Update opencode-autocode to the latest version
    Update,
    /// Initialize a new project (alias for interactive mode)
    Init {
        /// Use the default app spec template
        #[arg(long)]
        default: bool,

        /// Path to a custom app spec file
        #[arg(long, value_name = "FILE")]
        spec: Option<PathBuf>,

        /// Disable parallel subagent spec generation
        #[arg(long)]
        no_subagents: bool,
    },
}

/// Template subcommand actions
#[derive(Subcommand, Debug)]
pub enum TemplateAction {
    /// List all available templates
    List,
    /// Use a specific template
    Use {
        /// Name of the template to use
        name: String,
    },
}

/// Database subcommand actions
#[derive(Subcommand, Debug)]
pub enum DbAction {
    /// Initialize the SQLite database
    Init {
        /// Custom database path (default: .opencode.db)
        #[arg(long, value_name = "FILE")]
        path: Option<PathBuf>,
    },
    /// Import features from feature_list.json into the database
    Migrate {
        /// Path to feature_list.json (default: feature_list.json)
        #[arg(value_name = "FILE")]
        json_path: Option<PathBuf>,
    },
    /// Export features from database to JSON file
    Export {
        /// Output JSON file path (default: feature_list_export.json)
        #[arg(value_name = "FILE")]
        output: Option<PathBuf>,
    },
    /// Show database statistics
    Stats,
    /// Execute a SELECT query (read-only)
    Query {
        /// SQL SELECT query string
        sql: String,
    },
    /// Execute a write query (INSERT, UPDATE, DELETE, CREATE)
    Exec {
        /// SQL modification query string
        sql: String,
    },
    /// Run regression checks on current project features
    Check {
        /// Path to custom feature database or JSON (legacy)
        #[arg(long, value_name = "FILE")]
        path: Option<PathBuf>,
    },
    /// List all tables in the database
    Tables,
    /// Show schema for a table
    Schema {
        /// Table name to describe
        table: String,
    },
    /// Get the next incomplete feature
    NextFeature,
    /// Mark a feature as passing
    MarkPass {
        /// Feature ID to mark as passing
        id: i32,
    },
    /// Manage persistent agent knowledge
    Knowledge {
        #[command(subcommand)]
        action: KnowledgeAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum KnowledgeAction {
    /// Save a fact (key=value)
    Set {
        /// Unique key (e.g. dev_port)
        key: String,
        /// Value to store
        value: String,
        /// Category (default: general)
        #[arg(short, long)]
        category: Option<String>,
        /// Optional description
        #[arg(short, long)]
        description: Option<String>,
    },
    /// Get a fact by key
    Get { key: String },
    /// List all facts
    List {
        /// Filter by category
        #[arg(short, long)]
        category: Option<String>,
    },
    /// Delete a fact
    Delete { key: String },
    /// Track a server process (saves port→PID mapping)
    TrackServer {
        /// Port the server is running on
        port: u16,
        /// PID of the server process
        pid: u32,
    },
    /// Get the tracked PID for a server on a port
    GetServer {
        /// Port to look up
        port: u16,
    },
    /// Remove tracking for a server (use after killing it)
    UntrackServer {
        /// Port to untrack
        port: u16,
    },
}

/// Example topics for progressive discovery
#[derive(Subcommand, Debug)]
pub enum ExampleTopic {
    /// Show database-related examples
    Db {
        /// Show example feature INSERT statements
        #[arg(long)]
        insert: bool,
        /// Show example SQL queries for feature inspection
        #[arg(long)]
        query: bool,
    },
    /// Show example verification commands by project type
    Verify,
    /// Show example autocode.toml configuration sections
    Config,
    /// Show example conductor files (product.md, tech_stack.md)
    Conductor,
    /// Show the vibe loop workflow phases
    Workflow,
    /// Show app spec structure and sections
    Spec,
    /// Show agent identity and core values
    Identity,
    /// Show security constraints and allowlist usage
    Security,
    /// Show Model Context Protocol (MCP) tool guide
    Mcp,
    /// Show project architecture overview
    Arch,
    /// Show Rust development guide
    Rust,
    /// Show JavaScript/TypeScript development guide
    Js,
    /// Show testing strategies and E2E guide
    Testing,
    /// Show autonomous recovery protocols
    Recovery,
    /// Show high-speed orientation guide for autonomous agents
    Vibe,
    /// Show Conductor tracks and planning system guide
    Tracks,
    /// Show Interactive TUI spec generation workflow
    Interactive,
    /// Show Guide for project templates
    TemplatesGuide,
}

/// The mode of operation
pub enum Mode {
    /// Use the default embedded app spec
    Default,
    /// Use a custom app spec file
    Custom(PathBuf),
    /// Launch interactive TUI
    Interactive,
    /// Configure settings
    Config,
}

impl Cli {
    /// Determine the mode of operation based on CLI arguments
    pub fn mode(&self) -> Result<Mode> {
        // Check exclusive flags
        if self.config {
            return Ok(Mode::Config);
        }
        if self.default {
            return Ok(Mode::Default);
        }
        if let Some(ref spec_path) = self.spec {
            if !spec_path.exists() {
                bail!("Spec file not found: {}", spec_path.display());
            }
            return Ok(Mode::Custom(spec_path.clone()));
        }
        if self.interactive {
            return Ok(Mode::Interactive);
        }
        // Default to interactive
        Ok(Mode::Interactive)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_cli() -> Cli {
        Cli {
            command: None,
            interactive: false,
            default: false,
            spec: None,
            config: false,
            output: None,
            dry_run: false,
            no_subagents: false,
            verbose: false,
        }
    }

    #[test]
    fn test_default_mode() {
        let mut cli = default_cli();
        cli.default = true;
        assert!(matches!(cli.mode().unwrap(), Mode::Default));
    }

    #[test]
    fn test_interactive_mode() {
        let mut cli = default_cli();
        cli.interactive = true;
        assert!(matches!(cli.mode().unwrap(), Mode::Interactive));
    }

    #[test]
    fn test_config_mode() {
        let mut cli = default_cli();
        cli.config = true;
        assert!(matches!(cli.mode().unwrap(), Mode::Config));
    }
}
