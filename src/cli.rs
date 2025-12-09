//! CLI argument parsing using clap

use anyhow::{bail, Result};
use clap::Parser;
use std::path::PathBuf;

/// OpenCode Autonomous Coding Plugin Scaffolder
#[derive(Parser, Debug)]
#[command(name = "opencode-autocode")]
#[command(author = "CodingInCarhartts")]
#[command(version)]
#[command(about = "Scaffold autonomous coding plugin for OpenCode", long_about = None)]
pub struct Cli {
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

    /// Initialize git repository after scaffolding
    #[arg(long)]
    pub git_init: bool,
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
            default: true,
            spec: None,
            interactive: false,
            output: None,
            git_init: false,
        };
        assert!(matches!(cli.mode().unwrap(), Mode::Default));
    }

    #[test]
    fn test_interactive_mode() {
        let cli = Cli {
            default: false,
            spec: None,
            interactive: true,
            output: None,
            git_init: false,
        };
        assert!(matches!(cli.mode().unwrap(), Mode::Interactive));
    }
}
