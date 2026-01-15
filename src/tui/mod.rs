//! Interactive TUI for building app specs
//!
//! Provides multiple modes for creating project specifications:
//! - Generated: AI creates spec from idea
//! - Manual: Step-by-step form
//! - From file: Use existing spec
//! - Default: Built-in template
//!
//! # Modules
//!
//! - `actions`: TUI action handlers and event processing
//! - `components`: Reusable UI components
//! - `fullscreen`: Fullscreen TUI applications
//! - `generated`: AI-generated spec workflow
//! - `manual`: Manual spec creation workflow
//! - `prompts`: TUI prompt utilities
//! - `stats`: Statistics display components
//! - `theme`: Theming and styling utilities
//! - `validation`: Validation-related TUI components

mod actions;
pub mod components;
pub mod fullscreen;
pub mod generated;
pub mod manual;
pub mod prompts;
pub mod stats;
pub mod theme;
mod validation;

// Re-export the main entry points
pub use fullscreen::run_interactive;

// Re-export the config editor
pub use fullscreen::run_fullscreen_config_editor;
