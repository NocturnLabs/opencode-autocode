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

    /// Run regression checks on feature_list.json
    #[arg(long)]
    pub regression_check: bool,

    /// Output directory for scaffolded files
    #[arg(short, long, value_name = "DIR")]
    pub output: Option<PathBuf>,

    /// Preview scaffolding without creating files
    #[arg(long, visible_alias = "preview")]
    pub dry_run: bool,

    /// Path to feature_list.json (for --regression-check)
    #[arg(long, value_name = "FILE")]
    pub feature_list: Option<PathBuf>,

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
    },
    /// Manage project templates
    Templates {
        #[command(subcommand)]
        action: TemplateAction,
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
    /// Run regression checks
    RegressionCheck,
}

impl Cli {
    /// Determine the mode of operation based on CLI arguments
    pub fn mode(&self) -> Result<Mode> {
        // Check exclusive flags
        if self.config {
            return Ok(Mode::Config);
        }
        if self.regression_check {
            return Ok(Mode::RegressionCheck);
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
            regression_check: false,
            output: None,
            dry_run: false,
            feature_list: None,
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

    #[test]
    fn test_regression_check_mode() {
        let mut cli = default_cli();
        cli.regression_check = true;
        assert!(matches!(cli.mode().unwrap(), Mode::RegressionCheck));
    }
}
