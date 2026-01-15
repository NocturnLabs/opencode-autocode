use serde::{Deserialize, Serialize};

/// Interactive mode options
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug, Serialize, Deserialize)]
pub enum InteractiveMode {
    #[default]
    Generated,
    Manual,
    FromSpecFile,
    Default,
}

impl InteractiveMode {
    pub fn all() -> &'static [InteractiveMode] {
        &[
            InteractiveMode::Generated,
            InteractiveMode::Manual,
            InteractiveMode::FromSpecFile,
            InteractiveMode::Default,
        ]
    }

    /// Returns a unique identifier for this mode (used for IPC).
    pub fn id(&self) -> &'static str {
        match self {
            InteractiveMode::Generated => "generated",
            InteractiveMode::Manual => "manual",
            InteractiveMode::FromSpecFile => "from_spec_file",
            InteractiveMode::Default => "default",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            InteractiveMode::Generated => {
                "🤖 AI Generated - Let AI research and create a full spec"
            }
            InteractiveMode::Manual => "📝 Manual - Fill out project details step by step",
            InteractiveMode::FromSpecFile => "📁 From File - Use an existing spec file",
            InteractiveMode::Default => "⚡ Default - Use built-in specification",
        }
    }

    pub fn description(&self) -> &'static str {
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

/// Result from the fullscreen TUI
#[derive(Debug, Clone)]
pub struct SetupResult {
    pub mode: Option<InteractiveMode>,
    pub should_configure: bool,
    pub reconfigure: bool,
}
