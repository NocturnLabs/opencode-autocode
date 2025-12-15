//! CLI argument parsing using clap

use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// OpenCode Autonomous Coding Plugin Scaffolder
#[derive(Parser, Debug)]
#[command(name = "opencode-autocode")]
#[command(author = "CodingInCarhartts")]
#[command(version)]
#[command(about = "Scaffold autonomous coding plugin for OpenCode", long_about = None)]
pub struct Cli {
    /// Subcommand to run
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Use the default app spec template
    #[arg(short, long, conflicts_with_all = ["spec", "interactive"])]
    pub default: bool,

    /// Path to a custom app spec file
    #[arg(short, long, value_name = "FILE", conflicts_with_all = ["default", "interactive"])]
    pub spec: Option<PathBuf>,

    /// Launch interactive TUI to build app spec
    #[arg(short, long, conflicts_with_all = ["default", "spec"])]
    pub interactive: bool,

    /// Output directory for scaffolded files (default: current directory)
    #[arg(short, long, value_name = "DIR")]
    pub output: Option<PathBuf>,

    /// Preview mode: show what files would be created without creating them
    #[arg(long, visible_alias = "preview")]
    pub dry_run: bool,

    /// Initialize git repository after scaffolding
    #[arg(long)]
    pub git_init: bool,

    /// Path to custom config file (default: autocode.toml in current directory)
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,
}

/// Subcommands for the CLI
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Manage project templates
    Templates {
        #[command(subcommand)]
        action: TemplateAction,
    },
    /// Launch interactive spec editor (TUI)
    Edit,
    /// Run regression checks on feature_list.json
    RegressionCheck {
        /// Path to feature_list.json (defaults to ./feature_list.json)
        #[arg(short, long, value_name = "FILE")]
        feature_list: Option<PathBuf>,

        /// Only check features matching this category
        #[arg(short, long)]
        category: Option<String>,

        /// Verbose output showing each test
        #[arg(short, long)]
        verbose: bool,
    },
    /// Run the autonomous agent loop (replaces run-autonomous.sh)
    Autonomous {
        /// Maximum number of iterations (default: unlimited)
        #[arg(short, long)]
        limit: Option<usize>,

        /// Path to custom config file (default: autocode.toml)
        #[arg(short, long, value_name = "FILE")]
        config: Option<PathBuf>,
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

/// The mode of operation for the scaffolder
pub enum Mode {
    /// Use the default embedded app spec
    Default,
    /// Use a custom app spec file
    Custom(PathBuf),
    /// Launch interactive TUI
    Interactive,
}

impl Cli {
    /// Determine the mode of operation based on CLI arguments
    pub fn mode(&self) -> Result<Mode> {
        if self.default {
            Ok(Mode::Default)
        } else if let Some(ref spec_path) = self.spec {
            if !spec_path.exists() {
                bail!("Spec file not found: {}", spec_path.display());
            }
            Ok(Mode::Custom(spec_path.clone()))
        } else if self.interactive {
            Ok(Mode::Interactive)
        } else {
            // Default to interactive if no mode specified
            Ok(Mode::Interactive)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_mode() {
        let cli = Cli {
            command: None,
            default: true,
            spec: None,
            interactive: false,
            output: None,
            dry_run: false,
            git_init: false,
            config: None,
        };
        assert!(matches!(cli.mode().unwrap(), Mode::Default));
    }

    #[test]
    fn test_interactive_mode() {
        let cli = Cli {
            command: None,
            default: false,
            spec: None,
            interactive: true,
            output: None,
            dry_run: false,
            git_init: false,
            config: None,
        };
        assert!(matches!(cli.mode().unwrap(), Mode::Interactive));
    }
}
